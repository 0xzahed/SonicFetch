mod common;

use anyhow::Result;
use common::{Reply, fixture_server};
use sonicfetch_core::DownloadState;
use sonicfetch_engine::DownloadManager;
use sonicfetch_storage::{DownloadRecord, SqliteDownloadRepository};
use std::time::Duration;
use tokio::{io::AsyncReadExt, net::TcpListener, sync::broadcast};
use uuid::Uuid;

async fn setup() -> Result<(tempfile::TempDir, DownloadManager)> {
    let directory = tempfile::tempdir()?;
    let repository = SqliteDownloadRepository::open(directory.path().join("queue.sqlite3")).await?;
    let manager = DownloadManager::new(repository)?;
    Ok((directory, manager))
}

async fn wait_for_state(
    events: &mut broadcast::Receiver<sonicfetch_engine::DownloadEvent>,
    id: Uuid,
    expected: DownloadState,
) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let event = events.recv().await?;
            if event.id == id && event.state == expected { return Ok::<_, anyhow::Error>(()); }
            if event.id == id && event.state == DownloadState::Failed && expected != DownloadState::Failed {
                anyhow::bail!("download failed before reaching {expected:?}");
            }
        }
    }).await??;
    Ok(())
}

#[tokio::test]
async fn streams_http_body_to_the_selected_directory() -> Result<()> {
    let (directory, manager) = setup().await?;
    let (url, server) = fixture_server(vec![Reply::ok(b"fixture-payload".to_vec())]).await;
    let mut events = manager.subscribe();
    let download_url = url.join("payload.bin")?.to_string();
    let id = manager.add(&download_url, directory.path()).await?;
    wait_for_state(&mut events, id, DownloadState::Completed).await?;
    let requests = server.await?;

    let record = manager.list().await?.into_iter().find(|record| record.id == id).unwrap();
    assert_eq!(tokio::fs::read(record.final_path).await?, b"fixture-payload");
    assert!(requests[0].starts_with("GET /payload.bin HTTP/1.1"));
    Ok(())
}

#[tokio::test]
async fn follows_http_redirects() -> Result<()> {
    let (_directory, manager) = setup().await?;
    let (url, server) = fixture_server(vec![
        Reply { status: "302 Found", headers: "Location: /payload.bin\r\n", body: Vec::new() },
        Reply::ok(b"redirected".to_vec()),
    ]).await;
    let output = tempfile::tempdir()?;
    let mut events = manager.subscribe();
    let download_url = url.join("start")?.to_string();
    let id = manager.add(&download_url, output.path()).await?;
    wait_for_state(&mut events, id, DownloadState::Completed).await?;
    let requests = server.await?;

    assert_eq!(requests.len(), 2);
    assert!(requests[0].starts_with("GET /start HTTP/1.1"));
    assert!(requests[1].starts_with("GET /payload.bin HTTP/1.1"));
    let record = manager.list().await?.into_iter().find(|record| record.id == id).unwrap();
    assert_eq!(tokio::fs::read(record.final_path).await?, b"redirected");
    Ok(())
}

#[tokio::test]
async fn refuses_to_replace_existing_destination() -> Result<()> {
    let (directory, manager) = setup().await?;
    tokio::fs::write(directory.path().join("payload.bin"), b"keep-me").await?;
    let (url, server) = fixture_server(vec![Reply::ok(b"new-file".to_vec())]).await;
    let mut events = manager.subscribe();
    let download_url = url.join("payload.bin")?.to_string();
    let id = manager.add(&download_url, directory.path()).await?;
    wait_for_state(&mut events, id, DownloadState::Completed).await?;
    server.await?;

    let record = manager.list().await?.into_iter().find(|record| record.id == id).unwrap();
    assert_eq!(tokio::fs::read(directory.path().join("payload.bin")).await?, b"keep-me");
    assert_eq!(tokio::fs::read(record.final_path).await?, b"new-file");
    Ok(())
}

#[tokio::test]
async fn resumes_only_with_matching_range_and_validator() -> Result<()> {
    let (directory, manager) = setup().await?;
    let download_dir = directory.path().join("downloads");
    tokio::fs::create_dir_all(&download_dir).await?;
    let (url, server) = fixture_server(vec![Reply {
        status: "206 Partial Content",
        headers: "Content-Range: bytes 3-6/7\r\nETag: \"version-1\"\r\n",
        body: b"defg".to_vec(),
    }]).await;
    let id = Uuid::new_v4();
    let temp_path = download_dir.join(".payload.bin.partial");
    let final_path = download_dir.join("payload.bin");
    tokio::fs::write(&temp_path, b"abc").await?;
    let repo = SqliteDownloadRepository::open(directory.path().join("queue.sqlite3")).await?;
    repo.create(&DownloadRecord {
        id,
        source_url: url.join("payload.bin")?.to_string(),
        sanitized_filename: "payload.bin".into(),
        destination_dir: download_dir.to_string_lossy().into_owned(),
        temp_path: temp_path.to_string_lossy().into_owned(),
        final_path: final_path.to_string_lossy().into_owned(),
        state: DownloadState::Paused,
        downloaded_bytes: 3,
        total_bytes: Some(7),
        etag: Some("\"version-1\"".into()),
        last_modified: None,
        error_message: None,
        worker_pid: None,
    }).await?;

    let mut events = manager.subscribe();
    manager.resume(id).await?;
    wait_for_state(&mut events, id, DownloadState::Completed).await?;
    let requests = server.await?;
    assert!(requests[0].to_ascii_lowercase().contains("range: bytes=3-"));
    assert!(requests[0].to_ascii_lowercase().contains("if-range: \"version-1\""));
    assert_eq!(tokio::fs::read(final_path).await?, b"abcdefg");
    Ok(())
}

#[tokio::test]
async fn discards_partial_bytes_when_the_remote_etag_changes() -> Result<()> {
    let (directory, manager) = setup().await?;
    let download_dir = directory.path().join("downloads");
    tokio::fs::create_dir_all(&download_dir).await?;
    let (url, server) = fixture_server(vec![Reply {
        status: "206 Partial Content",
        headers: "Content-Range: bytes 3-5/6\r\nETag: \"version-2\"\r\n",
        body: b"def".to_vec(),
    }]).await;
    let id = Uuid::new_v4();
    let temp_path = download_dir.join(format!(".payload.bin.{id}.part"));
    tokio::fs::write(&temp_path, b"abc").await?;
    let repo = SqliteDownloadRepository::open(directory.path().join("queue.sqlite3")).await?;
    repo.create(&DownloadRecord {
        id,
        source_url: url.join("payload.bin")?.to_string(),
        sanitized_filename: "payload.bin".into(),
        destination_dir: download_dir.to_string_lossy().into_owned(),
        temp_path: temp_path.to_string_lossy().into_owned(),
        final_path: download_dir.join("payload.bin").to_string_lossy().into_owned(),
        state: DownloadState::Paused,
        downloaded_bytes: 3,
        total_bytes: Some(6),
        etag: Some("\"version-1\"".into()),
        last_modified: None,
        error_message: None,
        worker_pid: None,
    }).await?;

    let mut events = manager.subscribe();
    manager.resume(id).await?;
    wait_for_state(&mut events, id, DownloadState::Failed).await?;
    server.await?;
    assert!(tokio::fs::symlink_metadata(temp_path).await.is_err());
    Ok(())
}

#[tokio::test]
async fn restarts_from_zero_when_server_ignores_range_request() -> Result<()> {
    let (directory, manager) = setup().await?;
    let download_dir = directory.path().join("downloads");
    tokio::fs::create_dir_all(&download_dir).await?;
    let (url, server) = fixture_server(vec![Reply {
        status: "200 OK",
        headers: "ETag: \"version-1\"\r\n",
        body: b"abcdefg".to_vec(),
    }]).await;
    let id = Uuid::new_v4();
    let temp_path = download_dir.join(format!(".payload.bin.{id}.part"));
    let final_path = download_dir.join("payload.bin");
    tokio::fs::write(&temp_path, b"abc").await?;
    let repo = SqliteDownloadRepository::open(directory.path().join("queue.sqlite3")).await?;
    repo.create(&DownloadRecord {
        id,
        source_url: url.join("payload.bin")?.to_string(),
        sanitized_filename: "payload.bin".into(),
        destination_dir: download_dir.to_string_lossy().into_owned(),
        temp_path: temp_path.to_string_lossy().into_owned(),
        final_path: final_path.to_string_lossy().into_owned(),
        state: DownloadState::Paused,
        downloaded_bytes: 3,
        total_bytes: Some(7),
        etag: Some("\"version-1\"".into()),
        last_modified: None,
        error_message: None,
        worker_pid: None,
    }).await?;

    let mut events = manager.subscribe();
    manager.resume(id).await?;
    wait_for_state(&mut events, id, DownloadState::Completed).await?;
    let requests = server.await?;
    assert!(requests[0].to_ascii_lowercase().contains("range: bytes=3-"));
    assert_eq!(tokio::fs::read(final_path).await?, b"abcdefg");
    Ok(())
}

#[tokio::test]
async fn retries_a_transient_server_failure() -> Result<()> {
    let (directory, manager) = setup().await?;
    let (url, server) = fixture_server(vec![
        Reply { status: "500 Internal Server Error", headers: "", body: b"temporary failure".to_vec() },
        Reply::ok(b"recovered".to_vec()),
    ]).await;
    let download_url = url.join("retry.bin")?.to_string();
    let mut events = manager.subscribe();
    let id = manager.add(&download_url, directory.path()).await?;
    wait_for_state(&mut events, id, DownloadState::Failed).await?;
    manager.retry(id).await?;
    wait_for_state(&mut events, id, DownloadState::Completed).await?;
    let requests = server.await?;

    assert_eq!(requests.len(), 2);
    let record = manager.list().await?.into_iter().find(|record| record.id == id).unwrap();
    assert_eq!(tokio::fs::read(record.final_path).await?, b"recovered");
    Ok(())
}

#[tokio::test]
async fn rejects_unsupported_schemes_and_embedded_credentials() -> Result<()> {
    let (directory, manager) = setup().await?;
    assert!(manager.add("file:///etc/passwd", directory.path()).await.is_err());
    assert!(manager.add("ftp://example.test/file", directory.path()).await.is_err());
    assert!(manager.add("https://user:password@example.test/file", directory.path()).await.is_err());
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn rejects_a_symlink_as_the_partial_file() -> Result<()> {
    use std::os::unix::fs::symlink;

    let (directory, manager) = setup().await?;
    let download_dir = directory.path().join("downloads");
    tokio::fs::create_dir_all(&download_dir).await?;
    let victim = directory.path().join("victim");
    tokio::fs::write(&victim, b"do-not-truncate").await?;
    let id = Uuid::new_v4();
    let temp_path = download_dir.join(format!(".payload.bin.{id}.part"));
    symlink(&victim, &temp_path)?;

    let repo = SqliteDownloadRepository::open(directory.path().join("queue.sqlite3")).await?;
    repo.create(&DownloadRecord {
        id,
        source_url: "https://example.invalid/payload.bin".into(),
        sanitized_filename: "payload.bin".into(),
        destination_dir: download_dir.to_string_lossy().into_owned(),
        temp_path: temp_path.to_string_lossy().into_owned(),
        final_path: download_dir.join("payload.bin").to_string_lossy().into_owned(),
        state: DownloadState::Paused,
        downloaded_bytes: 0,
        total_bytes: None,
        etag: None,
        last_modified: None,
        error_message: None,
        worker_pid: None,
    }).await?;

    let mut events = manager.subscribe();
    manager.resume(id).await?;
    wait_for_state(&mut events, id, DownloadState::Failed).await?;
    assert_eq!(tokio::fs::read(victim).await?, b"do-not-truncate");
    Ok(())
}

#[tokio::test]
async fn cancellation_stops_a_worker_waiting_for_response_headers() -> Result<()> {
    let (directory, manager) = setup().await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await?;
        let mut request = [0_u8; 2048];
        let _ = stream.read(&mut request).await?;
        tokio::time::sleep(Duration::from_secs(30)).await;
        Ok::<_, std::io::Error>(())
    });

    let download_url = format!("http://{address}/stalled");
    let id = manager.add(&download_url, directory.path()).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    manager.cancel(id).await?;

    let record = manager.list().await?.into_iter().find(|record| record.id == id).unwrap();
    assert_eq!(record.state, DownloadState::Cancelled);
    assert!(record.worker_pid.is_none());
    server.abort();
    Ok(())
}
