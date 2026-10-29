use sonicfetch_core::DownloadState;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DownloadRecord {
    pub id: Uuid,
    pub source_url: String,
    pub sanitized_filename: String,
    pub destination_dir: String,
    pub state: DownloadState,
    pub downloaded_bytes: i64,
    pub total_bytes: Option<i64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SegmentRecord {
    pub id: Uuid,
    pub download_id: Uuid,
    pub segment_index: i64,
    pub range_start: i64,
    pub range_end: i64,
    pub bytes_written: i64,
}
