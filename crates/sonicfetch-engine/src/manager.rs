use anyhow::{Context, Result, bail};
use reqwest::{header, Client, StatusCode};
use sonicfetch_core::DownloadState;
use sonicfetch_storage::{DownloadRecord, SqliteDownloadRepository};
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc, time::Duration};
use tokio::{io::AsyncWriteExt, sync::{broadcast, Mutex}};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DownloadEvent { pub id: Uuid, pub state: DownloadState, pub at: chrono::DateTime<chrono::Utc>, pub bytes: Option<u64>, pub total: Option<u64> }

#[derive(Clone)]
pub struct DownloadManager {
    repo: SqliteDownloadRepository,
    client: Client,
    events: broadcast::Sender<DownloadEvent>,
    active: Arc<Mutex<HashMap<Uuid, CancellationToken>>>,
}

impl DownloadManager {
    pub fn new(repo: SqliteDownloadRepository) -> Result<Self> {
        let (events, _) = broadcast::channel(1024);
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(20))
            .read_timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 10 {
                    return attempt.error("too many redirects");
                }
                if attempt.url().username() != "" || attempt.url().password().is_some() {
                    return attempt.error("redirects containing credentials are not allowed");
                }
                if attempt.previous().last().is_some_and(|previous| previous.scheme() == "https")
                    && attempt.url().scheme() != "https"
                {
                    return attempt.error("HTTPS redirects may not downgrade to HTTP");
                }
                attempt.follow()
            }))
            .user_agent(concat!("SonicFetch/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { repo, client, events, active: Arc::new(Mutex::new(HashMap::new())) })
    }
    pub fn subscribe(&self) -> broadcast::Receiver<DownloadEvent> { self.events.subscribe() }
    pub async fn list(&self) -> Result<Vec<DownloadRecord>> { self.repo.list().await }
    fn emit(&self, id: Uuid, state: DownloadState, bytes: Option<u64>, total: Option<u64>) {
        let _ = self.events.send(DownloadEvent { id, state, at: chrono::Utc::now(), bytes, total });
    }

    pub async fn add(&self, input: &str, directory: &Path) -> Result<Uuid> {
        let parsed = url::Url::parse(input).context("invalid URL")?;
        if !matches!(parsed.scheme(), "http" | "https") || !parsed.username().is_empty() || parsed.password().is_some() { bail!("only credential-free HTTP and HTTPS URLs are supported"); }
        let mut filename = parsed.path_segments().and_then(|mut parts| parts.next_back()).filter(|s| !s.is_empty()).unwrap_or("download").to_string();
        filename = filename.chars().filter(|c| !c.is_control() && !matches!(c, '/' | '\\')).take(180).collect();
        if filename.is_empty() || filename == "." || filename == ".." { filename = "download".into(); }
        tokio::fs::create_dir_all(directory).await?;
        let destination = std::fs::canonicalize(directory)?;
        let id = Uuid::new_v4();
        let final_path = unique_destination(&destination, &filename);
        let temp_path = destination.join(format!(".{filename}.{id}.part"));
        let record = DownloadRecord { id, source_url: parsed.to_string(), sanitized_filename: filename,
            destination_dir: destination.to_string_lossy().into_owned(), temp_path: temp_path.to_string_lossy().into_owned(),
            final_path: final_path.to_string_lossy().into_owned(), state: DownloadState::Queued,
            downloaded_bytes: 0, total_bytes: None, etag: None, last_modified: None, error_message: None, worker_pid: None };
        self.repo.create(&record).await?;
        self.emit(id, DownloadState::Queued, Some(0), None);
        self.launch(id).await?;
        Ok(id)
    }

    pub async fn pause(&self, id: Uuid) -> Result<()> {
        let current = self.repo.get(id).await?.context("download not found")?;
        if !matches!(current.state, DownloadState::Queued | DownloadState::Preparing | DownloadState::Downloading | DownloadState::Paused) {
            bail!("download cannot be paused in its current state");
        }
        if !self.repo.request_pause(id).await? { bail!("download changed state; try again"); }
        if let Some(token) = self.active.lock().await.get(&id) { token.cancel(); }
        self.wait_for_worker_stop(id).await?;
        self.emit(id, DownloadState::Paused, None, None);
        Ok(())
    }
    pub async fn cancel(&self, id: Uuid) -> Result<()> {
        self.repo.get(id).await?.context("download not found")?;
        if !self.repo.request_cancel(id).await? { bail!("download cannot be cancelled in its current state"); }
        if let Some(token) = self.active.lock().await.get(&id) { token.cancel(); }
        self.wait_for_worker_stop(id).await?;
        self.emit(id, DownloadState::Cancelled, None, None);
        Ok(())
    }
    async fn wait_for_worker_stop(&self, id: Uuid) -> Result<()> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            self.repo.recover_interrupted().await?;
            let record = self.repo.get(id).await?.context("download not found")?;
            if record.worker_pid.is_none() { return Ok(()); }
            if tokio::time::Instant::now() >= deadline { bail!("download worker did not stop; try again shortly"); }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    pub async fn resume(&self, id: Uuid) -> Result<()> { self.launch(id).await }
    pub async fn retry(&self, id: Uuid) -> Result<()> {
        let mut rec = self.repo.get(id).await?.context("download not found")?;
        if rec.worker_pid.is_some() {
            let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
            while rec.worker_pid.is_some() && tokio::time::Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(100)).await;
                self.repo.recover_interrupted().await?;
                rec = self.repo.get(id).await?.context("download was removed")?;
            }
            if rec.worker_pid.is_some() { bail!("download worker has not stopped yet; try again shortly"); }
        }
        if rec.state != DownloadState::Failed && rec.state != DownloadState::Cancelled { bail!("only failed or cancelled downloads can be retried"); }
        if !self.repo.transition(id, rec.state, DownloadState::Retrying).await? {
            bail!("download changed state; try retrying again");
        }
        self.launch(id).await
    }
    pub async fn remove(&self, id: Uuid, delete_file: bool) -> Result<bool> {
        let original = self.repo.get(id).await?.context("download not found")?;
        let (temp_path, final_path) = validate_record_paths(&original)?;
        if original.state == DownloadState::Completed {
            if delete_file { tokio::fs::remove_file(&final_path).await?; }
            return self.repo.remove(id).await;
        }
        if original.state == DownloadState::Verifying { bail!("download is finalizing; remove it after it completes"); }
        if !self.repo.request_cancel(id).await? { bail!("download changed state; try removing it again"); }
        if let Some(token) = self.active.lock().await.get(&id) { token.cancel(); }
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            self.repo.recover_interrupted().await?;
            let record = self.repo.get(id).await?.context("download not found")?;
            if record.worker_pid.is_none() { break; }
            if tokio::time::Instant::now() >= deadline { bail!("download worker did not stop; try removing it again shortly"); }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let _ = tokio::fs::remove_file(&temp_path).await;
        self.repo.remove(id).await
    }
    async fn launch(&self, id: Uuid) -> Result<()> {
        let mut active = self.active.lock().await;
        if active.contains_key(&id) { bail!("download is already running"); }
        let mut rec = self.repo.get(id).await?.context("download not found")?;
        if rec.state == DownloadState::Completed { bail!("download is already complete"); }
        if rec.state == DownloadState::Paused && rec.worker_pid.is_some() {
            let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
            while rec.worker_pid.is_some() && tokio::time::Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(100)).await;
                self.repo.recover_interrupted().await?;
                rec = self.repo.get(id).await?.context("download was removed")?;
            }
            if rec.worker_pid.is_some() { bail!("paused worker has not stopped yet; try again shortly"); }
        }
        let pid = std::process::id();
        if !matches!(rec.state, DownloadState::Queued | DownloadState::Paused | DownloadState::Retrying) {
            bail!("download must be queued or paused before it can start");
        }
        if !self.repo.claim_worker(id, pid).await? { bail!("download is already being handled by another process"); }
        let token = CancellationToken::new(); active.insert(id, token.clone());
        let manager = self.clone();
        tokio::spawn(async move {
            let outcome = manager.transfer(rec.clone(), token.clone()).await;
            match outcome {
                Ok(()) => {
                    if manager.repo.transition(id, DownloadState::Verifying, DownloadState::Completed).await.unwrap_or(false) {
                        manager.emit(id, DownloadState::Completed, None, None);
                    }
                }
                Err(_error) => {
                    let safe = redact_url(&rec.source_url);
                    if manager.repo.fail_active(id, "Transfer failed. Check connectivity, server response, and destination permissions.").await.unwrap_or(false) {
                        tracing::warn!(download_id=%id, url=%safe, "download failed");
                        manager.emit(id, DownloadState::Failed, None, None);
                    } else {
                        match manager.repo.get(id).await.ok().flatten().map(|record| record.state) {
                            Some(state @ (DownloadState::Paused | DownloadState::Cancelled)) => manager.emit(id, state, None, None),
                            _ => tracing::debug!(download_id=%id, "download stopped after a state change"),
                        }
                    }
                }
            }
            manager.active.lock().await.remove(&id);
            let _ = manager.repo.release_worker(id, pid).await;
        });
        Ok(())
    }
    async fn transfer(&self, rec: DownloadRecord, cancel: CancellationToken) -> Result<()> {
        self.emit(rec.id, DownloadState::Preparing, None, None);
        let (temp, final_path) = validate_record_paths(&rec)?;
        let existing_temp = match tokio::fs::symlink_metadata(&temp).await {
            Ok(metadata) => {
                validate_regular_single_link(&metadata)?;
                Some(metadata)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let current = existing_temp.as_ref().map_or(0, std::fs::Metadata::len);
        let mut request = self.client.get(&rec.source_url);
        if current > 0 && (rec.etag.is_some() || rec.last_modified.is_some()) {
            request = request.header(header::RANGE, format!("bytes={current}-"));
            if let Some(etag) = &rec.etag { request = request.header(header::IF_RANGE, etag); }
            else if let Some(modified) = &rec.last_modified { request = request.header(header::IF_RANGE, modified); }
        }
        let mut connect_poll = tokio::time::interval(Duration::from_millis(250));
        connect_poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let request_future = request.send();
        tokio::pin!(request_future);
        let response = loop {
            tokio::select! {
                _ = cancel.cancelled() => bail!("transfer stopped"),
                _ = connect_poll.tick() => match self.repo.get(rec.id).await?.map(|d| d.state) {
                    Some(DownloadState::Paused | DownloadState::Cancelled) => bail!("transfer stopped"),
                    None => bail!("download was removed"),
                    _ => continue,
                },
                result = &mut request_future => break result?,
            }
        };
        let status = response.status();
        if status == StatusCode::RANGE_NOT_SATISFIABLE && current > 0 {
            drop(response);
            let _ = tokio::fs::remove_file(&temp).await;
            bail!("partial file is no longer valid; it was discarded, retry to restart");
        }
        if !status.is_success() { bail!("server returned HTTP {status}"); }
        let append = current > 0 && status == StatusCode::PARTIAL_CONTENT;
        if status == StatusCode::PARTIAL_CONTENT && !append {
            bail!("server returned a partial response to a full download request");
        }
        let range_details = if append {
            response.headers().get(header::CONTENT_RANGE).and_then(|v| v.to_str().ok()).and_then(parse_content_range)
        } else { None };
        if append {
            let validator_matches = if let Some(saved) = &rec.etag {
                response.headers().get(header::ETAG).and_then(|v|v.to_str().ok()) == Some(saved.as_str())
            } else if let Some(saved) = &rec.last_modified {
                response.headers().get(header::LAST_MODIFIED).and_then(|v|v.to_str().ok()) == Some(saved.as_str())
            } else { false };
            let range_valid = range_details.is_some_and(|(start, end, total)| {
                let total_matches = match (total, rec.total_bytes) {
                    (Some(total), Some(saved)) => end.checked_add(1) == Some(total) && i64::try_from(total).ok() == Some(saved),
                    (Some(total), None) => end.checked_add(1) == Some(total),
                    (None, None) => true,
                    (None, Some(_)) => false,
                };
                start == current && end >= start && total_matches
                    && response.content_length().is_none_or(|len| end.checked_sub(start).and_then(|d| d.checked_add(1)) == Some(len))
            });
            if !range_valid || !validator_matches {
                drop(response);
                let _ = tokio::fs::remove_file(&temp).await;
                bail!("resume validator or content range changed; partial file discarded, retry to restart");
            }
        }
        let etag = response.headers().get(header::ETAG).and_then(|v|v.to_str().ok()).map(str::to_owned);
        let modified = response.headers().get(header::LAST_MODIFIED).and_then(|v|v.to_str().ok()).map(str::to_owned);
        let length = if append {
            range_details.and_then(|(_, _, total)| total).and_then(|total| i64::try_from(total).ok())
        } else {
            response.content_length().and_then(|n| i64::try_from(n).ok())
        };
        if !self.repo.transition(rec.id, DownloadState::Preparing, DownloadState::Downloading).await? {
            bail!("download state changed before transfer started");
        }
        self.repo.update_progress(rec.id, if append { current as i64 } else { 0 }, length, etag.as_deref(), modified.as_deref()).await?;
        self.emit(rec.id, DownloadState::Downloading, Some(if append {current}else{0}), length.map(|n|n as u64));
        let path_metadata_before_open = match tokio::fs::symlink_metadata(&temp).await {
            Ok(metadata) => { validate_regular_single_link(&metadata)?; Some(metadata) }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        match (&existing_temp, &path_metadata_before_open) {
            (None, None) => {}
            (Some(before), Some(now)) if same_file(before, now) => {}
            _ => bail!("temporary file changed while preparing transfer"),
        }
        let exists_now = path_metadata_before_open.is_some();
        let mut file_options = tokio::fs::OpenOptions::new();
        file_options.write(true).append(append).create_new(!exists_now);
        #[cfg(unix)] {
            file_options.custom_flags(libc::O_NOFOLLOW);
            file_options.mode(0o600);
        }
        let mut file = file_options.open(&temp).await?;
        let opened_metadata = file.metadata().await?;
        let path_metadata = tokio::fs::symlink_metadata(&temp).await?;
        validate_same_regular_file(&opened_metadata, &path_metadata)?;
        if append && opened_metadata.len() != current { bail!("partial file changed while preparing resume"); }
        if !append { file.set_len(0).await?; }
        let mut stream = response.bytes_stream();
        let mut bytes = if append { current } else { 0 };
        let mut control_poll = tokio::time::interval(Duration::from_millis(250));
        control_poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            let next = tokio::select! {
                _ = cancel.cancelled() => bail!("transfer stopped"),
                _ = control_poll.tick() => {
                    match self.repo.get(rec.id).await?.map(|d| d.state) {
                        Some(DownloadState::Paused | DownloadState::Cancelled) => bail!("transfer stopped"),
                        None => bail!("download was removed"),
                        _ => continue,
                    }
                },
                item = stream.next() => item,
            };
            let Some(chunk) = next else { break; };
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            bytes += chunk.len() as u64;
            self.repo.update_progress(rec.id, bytes as i64, length, etag.as_deref(), modified.as_deref()).await?;
            self.emit(rec.id, DownloadState::Downloading, Some(bytes), length.map(|n|n as u64));
        }
        file.flush().await?; file.sync_all().await?;
        let open_metadata = file.metadata().await?;
        let final_temp_metadata = tokio::fs::symlink_metadata(&temp).await?;
        validate_same_regular_file(&open_metadata, &final_temp_metadata)?;
        match self.repo.get(rec.id).await?.map(|d| d.state) {
            Some(DownloadState::Paused | DownloadState::Cancelled) => bail!("transfer stopped"),
            None => bail!("download was removed"),
            _ => {}
        }
        if let Some(total) = length { if bytes != total as u64 { bail!("incomplete response: received {bytes} of {total} bytes"); } }
        if !self.repo.transition(rec.id, DownloadState::Downloading, DownloadState::Verifying).await? {
            bail!("download state changed before finalization");
        }
        self.emit(rec.id, DownloadState::Verifying, Some(bytes), length.map(|n|n as u64));
        tokio::fs::hard_link(&temp, &final_path).await
            .context("destination already exists or does not support safe same-directory finalization")?;
        if let Some(parent) = final_path.parent() {
            tokio::fs::File::open(parent).await?.sync_all().await?;
        }
        tokio::fs::remove_file(&temp).await?;
        if let Some(parent) = final_path.parent() {
            tokio::fs::File::open(parent).await?.sync_all().await?;
        }
        Ok(())
    }
}

fn unique_destination(dir: &Path, filename: &str) -> PathBuf {
    let original = dir.join(filename);
    if !original.exists() { return original; }
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s|s.to_str()).unwrap_or("download");
    let ext = path.extension().and_then(|s|s.to_str());
    for n in 1..100_000 { let name = match ext { Some(ext)=>format!("{stem} ({n}).{ext}"),None=>format!("{stem} ({n})") }; let candidate=dir.join(name); if !candidate.exists(){return candidate;} }
    dir.join(format!("{Uuid::new_v4()}-{filename}"))
}

fn validate_record_paths(record: &DownloadRecord) -> Result<(PathBuf, PathBuf)> {
    if record.sanitized_filename.is_empty()
        || record.sanitized_filename.chars().any(|c| c.is_control() || matches!(c, '/' | '\\'))
    {
        bail!("download record contains an unsafe filename");
    }
    let directory = std::fs::canonicalize(&record.destination_dir)?;
    let temp = PathBuf::from(&record.temp_path);
    let expected_temp = directory.join(format!(".{}.{}.part", record.sanitized_filename, record.id));
    let final_path = PathBuf::from(&record.final_path);
    if temp != expected_temp || final_path.parent() != Some(directory.as_path()) || final_path.file_name().is_none() {
        bail!("download record paths do not match its destination directory");
    }
    Ok((temp, final_path))
}

fn validate_regular_single_link(metadata: &std::fs::Metadata) -> Result<()> {
    if !metadata.file_type().is_file() { bail!("temporary path is not a regular file"); }
    #[cfg(unix)] {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 { bail!("temporary file has unexpected hard links"); }
    }
    Ok(())
}

fn validate_same_regular_file(opened: &std::fs::Metadata, path: &std::fs::Metadata) -> Result<()> {
    validate_regular_single_link(opened)?;
    validate_regular_single_link(path)?;
    if !same_file(opened, path) { bail!("temporary file path changed while open"); }
    Ok(())
}

#[cfg(unix)]
fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file(_left: &std::fs::Metadata, _right: &std::fs::Metadata) -> bool { false }

fn parse_content_range(value: &str) -> Option<(u64, u64, Option<u64>)> {
    let value = value.strip_prefix("bytes ")?;
    let (range, total) = value.split_once('/')?;
    let (start, end) = range.split_once('-')?;
    Some((start.parse().ok()?, end.parse().ok()?, if total == "*" { None } else { Some(total.parse().ok()?) }))
}
fn redact_url(raw: &str) -> String {
    let Ok(mut u)=url::Url::parse(raw) else{return "<invalid-url>".into()};
    if u.password().is_some(){let _=u.set_password(Some("REDACTED"));}
    let keys:Vec<String>=u.query_pairs().map(|(k,_)|k.into_owned()).filter(|k|{let k=k.to_ascii_lowercase();["token","signature","sig","key","auth","credential","expires","x-amz-"] .iter().any(|needle|k.contains(needle))}).collect();
    for key in keys { let pairs:Vec<(String,String)>=u.query_pairs().map(|(k,v)|(k.into_owned(),if k.eq_ignore_ascii_case(&key){"REDACTED".into()}else{v.into_owned()})).collect(); u.set_query(None); if !pairs.is_empty(){u.query_pairs_mut().extend_pairs(pairs);} }
    u.to_string()
}

use futures_util::StreamExt;
