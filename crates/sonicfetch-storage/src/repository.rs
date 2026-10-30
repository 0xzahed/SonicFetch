use anyhow::Result;
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, Row, SqlitePool};
use std::time::Duration;
use uuid::Uuid;

use crate::models::DownloadRecord;

/// SQLite-backed queue storage. The database is created with private, local-file defaults.
#[derive(Clone)]
pub struct SqliteDownloadRepository { pool: SqlitePool }

impl SqliteDownloadRepository {
    pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            tokio::fs::create_dir_all(parent).await?;
            #[cfg(unix)] {
                use std::os::unix::fs::PermissionsExt;
                tokio::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).await?;
            }
        }
        let options = SqliteConnectOptions::new().filename(path).create_if_missing(true).foreign_keys(true);
        let pool = SqlitePoolOptions::new().max_connections(5).acquire_timeout(Duration::from_secs(10)).connect_with(options).await?;
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
        }
        sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
        for statement in include_str!("../../../migrations/0001_initial.sql").split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(statement).execute(&pool).await?;
        }
        let columns = sqlx::query("PRAGMA table_info(downloads)").fetch_all(&pool).await?;
        if !columns.iter().any(|row| row.try_get::<String, _>("name").is_ok_and(|name| name == "worker_pid")) {
            sqlx::query("ALTER TABLE downloads ADD COLUMN worker_pid INTEGER").execute(&pool).await?;
        }
        let repository = Self { pool };
        repository.recover_interrupted().await?;
        Ok(repository)
    }

    pub async fn create(&self, d: &DownloadRecord) -> Result<()> {
        sqlx::query("INSERT INTO downloads(id,source_url,sanitized_filename,destination_dir,temp_path,final_path,state,downloaded_bytes,total_bytes,etag,last_modified,worker_pid,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(d.id.to_string()).bind(&d.source_url).bind(&d.sanitized_filename).bind(&d.destination_dir)
            .bind(&d.temp_path).bind(&d.final_path).bind(state_name(d.state)).bind(d.downloaded_bytes)
            .bind(d.total_bytes).bind(&d.etag).bind(&d.last_modified).bind(d.worker_pid).bind(timestamp()).bind(timestamp())
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<DownloadRecord>> {
        let row = sqlx::query("SELECT * FROM downloads WHERE id=?").bind(id.to_string()).fetch_optional(&self.pool).await?;
        row.map(record_from_row).transpose()
    }

    pub async fn list(&self) -> Result<Vec<DownloadRecord>> {
        let rows = sqlx::query("SELECT * FROM downloads ORDER BY created_at DESC").fetch_all(&self.pool).await?;
        rows.into_iter().map(record_from_row).collect()
    }

    pub async fn update_progress(&self, id: Uuid, bytes: i64, total: Option<i64>, etag: Option<&str>, modified: Option<&str>) -> Result<()> {
        sqlx::query("UPDATE downloads SET downloaded_bytes=?,total_bytes=?,etag=?,last_modified=?,updated_at=? WHERE id=? AND state NOT IN ('Paused','Cancelled')")
            .bind(bytes).bind(total).bind(etag).bind(modified).bind(timestamp()).bind(id.to_string()).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn transition(&self, id: Uuid, from: sonicfetch_core::DownloadState, to: sonicfetch_core::DownloadState) -> Result<bool> {
        sonicfetch_core::StateTransition { from, to }.validate()?;
        Ok(sqlx::query("UPDATE downloads SET state=?,updated_at=?,completed_at=CASE WHEN ?='Completed' THEN ? ELSE completed_at END WHERE id=? AND state=?")
            .bind(state_name(to)).bind(timestamp()).bind(state_name(to)).bind(timestamp()).bind(id.to_string()).bind(state_name(from))
            .execute(&self.pool).await?.rows_affected() == 1)
    }

    pub async fn request_pause(&self, id: Uuid) -> Result<bool> {
        use sonicfetch_core::DownloadState::*;
        for from in [Queued, Preparing, Downloading] {
            sonicfetch_core::StateTransition { from, to: Paused }.validate()?;
        }
        let changed = sqlx::query("UPDATE downloads SET state='Paused',updated_at=? WHERE id=? AND state IN ('Queued','Preparing','Downloading')")
            .bind(timestamp()).bind(id.to_string()).execute(&self.pool).await?.rows_affected();
        if changed == 1 { return Ok(true); }
        Ok(self.get(id).await?.is_some_and(|record| record.state == Paused))
    }

    pub async fn request_cancel(&self, id: Uuid) -> Result<bool> {
        use sonicfetch_core::DownloadState::*;
        for from in [Queued, Preparing, Downloading, Paused, Retrying, Failed, NeedsAttention] {
            sonicfetch_core::StateTransition { from, to: Cancelled }.validate()?;
        }
        let changed = sqlx::query("UPDATE downloads SET state='Cancelled',updated_at=? WHERE id=? AND state IN ('Queued','Preparing','Downloading','Paused','Retrying','Failed','NeedsAttention')")
            .bind(timestamp()).bind(id.to_string()).execute(&self.pool).await?.rows_affected();
        if changed == 1 { return Ok(true); }
        Ok(self.get(id).await?.is_some_and(|record| record.state == Cancelled))
    }

    pub async fn fail_active(&self, id: Uuid, message: &str) -> Result<bool> {
        use sonicfetch_core::DownloadState::*;
        for from in [Preparing, Downloading, Verifying] {
            sonicfetch_core::StateTransition { from, to: Failed }.validate()?;
        }
        Ok(sqlx::query("UPDATE downloads SET state='Failed',error_message=?,updated_at=? WHERE id=? AND state IN ('Preparing','Downloading','Verifying')")
            .bind(message).bind(timestamp()).bind(id.to_string()).execute(&self.pool).await?.rows_affected() == 1)
    }

    pub async fn remove(&self, id: Uuid) -> Result<bool> {
        Ok(sqlx::query("DELETE FROM downloads WHERE id=?").bind(id.to_string()).execute(&self.pool).await?.rows_affected() > 0)
    }
}
fn timestamp() -> String { chrono::Utc::now().to_rfc3339() }
fn state_name(state: sonicfetch_core::DownloadState) -> &'static str {
    use sonicfetch_core::DownloadState::*;
    match state { Queued=>"Queued", Preparing=>"Preparing", Downloading=>"Downloading", Paused=>"Paused", Verifying=>"Verifying", Completed=>"Completed", Retrying=>"Retrying", Failed=>"Failed", Cancelled=>"Cancelled", NeedsAttention=>"NeedsAttention" }
}
fn parse_state(s: &str) -> anyhow::Result<sonicfetch_core::DownloadState> {
    use sonicfetch_core::DownloadState::*;
    Ok(match s { "Queued"=>Queued,"Preparing"=>Preparing,"Downloading"=>Downloading,"Paused"=>Paused,"Verifying"=>Verifying,"Completed"=>Completed,"Retrying"=>Retrying,"Failed"=>Failed,"Cancelled"=>Cancelled,"NeedsAttention"=>NeedsAttention,_=>anyhow::bail!("unknown persisted state {s}") })
}
fn record_from_row(r: sqlx::sqlite::SqliteRow) -> Result<DownloadRecord> {
    Ok(DownloadRecord {
        id: Uuid::parse_str(r.try_get("id")?)?, source_url: r.try_get("source_url")?,
        sanitized_filename: r.try_get("sanitized_filename")?, destination_dir: r.try_get("destination_dir")?,
        temp_path: r.try_get("temp_path")?, final_path: r.try_get("final_path")?,
        state: parse_state(r.try_get("state")?)?, downloaded_bytes: r.try_get("downloaded_bytes")?,
        total_bytes: r.try_get("total_bytes")?, etag: r.try_get("etag")?, last_modified: r.try_get("last_modified")?,
        error_message: r.try_get("error_message")?, worker_pid: r.try_get("worker_pid")?,
    })
}

impl SqliteDownloadRepository {
    pub async fn claim_worker(&self, id: Uuid, pid: u32) -> Result<bool> {
        use sonicfetch_core::DownloadState::*;
        for from in [Queued, Paused, Retrying] {
            sonicfetch_core::StateTransition { from, to: Preparing }.validate()?;
        }
        Ok(sqlx::query("UPDATE downloads SET worker_pid=?,state='Preparing',error_message=NULL,updated_at=? WHERE id=? AND worker_pid IS NULL AND state IN ('Queued','Paused','Retrying')")
            .bind(i64::from(pid)).bind(timestamp()).bind(id.to_string()).execute(&self.pool).await?.rows_affected() == 1)
    }

    pub async fn release_worker(&self, id: Uuid, pid: u32) -> Result<()> {
        sqlx::query("UPDATE downloads SET worker_pid=NULL,updated_at=? WHERE id=? AND worker_pid=?")
            .bind(timestamp()).bind(id.to_string()).bind(i64::from(pid)).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn recover_interrupted(&self) -> Result<()> {
        let rows = sqlx::query("SELECT id,state,worker_pid,temp_path,final_path FROM downloads WHERE worker_pid IS NOT NULL OR state IN ('Preparing','Downloading','Retrying','Verifying')")
            .fetch_all(&self.pool).await?;
        for row in rows {
            let id: String = row.try_get("id")?;
            let state: String = row.try_get("state")?;
            let pid: Option<i64> = row.try_get("worker_pid")?;
            let alive = pid.is_some_and(|pid| pid > 0 && std::path::Path::new(&format!("/proc/{pid}")).exists());
            if !alive {
                let recovered_state = if state == "Verifying" {
                    let temp: String = row.try_get("temp_path")?;
                    let final_path: String = row.try_get("final_path")?;
                    match (std::fs::metadata(&temp), std::fs::metadata(&final_path)) {
                        (Err(temp_error), Ok(_)) if temp_error.kind() == std::io::ErrorKind::NotFound => "Completed",
                        (Ok(temp_meta), Ok(final_meta)) if same_file(&temp_meta, &final_meta) => {
                            let _ = std::fs::remove_file(&temp);
                            "Completed"
                        }
                        (_, Ok(_)) => "NeedsAttention",
                        _ => "Queued",
                    }
                } else if matches!(state.as_str(), "Preparing" | "Downloading" | "Retrying" | "Verifying") { "Queued" } else { state.as_str() };
                sqlx::query("UPDATE downloads SET state=?,worker_pid=NULL,updated_at=?,error_message=CASE WHEN ?='NeedsAttention' THEN 'Final file exists but could not be verified against the temporary file.' ELSE NULL END,completed_at=CASE WHEN ?='Completed' THEN ? ELSE completed_at END WHERE id=?")
                    .bind(recovered_state).bind(timestamp()).bind(recovered_state).bind(recovered_state).bind(timestamp()).bind(id)
                    .execute(&self.pool).await?;
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file(_left: &std::fs::Metadata, _right: &std::fs::Metadata) -> bool { false }

#[cfg(test)]
mod tests {
    use super::*;
    use sonicfetch_core::DownloadState;

    fn record(root: &std::path::Path, state: DownloadState, worker_pid: Option<i64>) -> DownloadRecord {
        let id = Uuid::new_v4();
        DownloadRecord {
            id,
            source_url: "https://example.test/file.bin".into(),
            sanitized_filename: "file.bin".into(),
            destination_dir: root.to_string_lossy().into_owned(),
            temp_path: root.join(format!(".file.bin.{id}.part")).to_string_lossy().into_owned(),
            final_path: root.join("file.bin").to_string_lossy().into_owned(),
            state,
            downloaded_bytes: 0,
            total_bytes: None,
            etag: None,
            last_modified: None,
            error_message: None,
            worker_pid,
        }
    }

    #[tokio::test]
    async fn restores_a_download_owned_by_a_dead_process() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let repository = SqliteDownloadRepository::open(directory.path().join("queue.sqlite3")).await?;
        let item = record(directory.path(), DownloadState::Downloading, Some(i64::from(u32::MAX)));
        let id = item.id;
        repository.create(&item).await?;

        repository.recover_interrupted().await?;
        let restored = repository.get(id).await?.expect("record should remain");
        assert_eq!(restored.state, DownloadState::Queued);
        assert_eq!(restored.worker_pid, None);
        Ok(())
    }

    #[tokio::test]
    async fn recognizes_a_file_published_before_crash() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let repository = SqliteDownloadRepository::open(directory.path().join("queue.sqlite3")).await?;
        let item = record(directory.path(), DownloadState::Verifying, None);
        let id = item.id;
        tokio::fs::write(&item.final_path, b"complete").await?;
        repository.create(&item).await?;

        repository.recover_interrupted().await?;
        let restored = repository.get(id).await?.expect("record should remain");
        assert_eq!(restored.state, DownloadState::Completed);
        Ok(())
    }

    #[tokio::test]
    async fn rejects_invalid_lifecycle_transition() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let repository = SqliteDownloadRepository::open(directory.path().join("queue.sqlite3")).await?;
        let item = record(directory.path(), DownloadState::Queued, None);
        let id = item.id;
        repository.create(&item).await?;

        assert!(repository.transition(id, DownloadState::Queued, DownloadState::Completed).await.is_err());
        assert_eq!(repository.get(id).await?.expect("record should remain").state, DownloadState::Queued);
        Ok(())
    }
}
