use anyhow::Result;
use uuid::Uuid;

use crate::models::DownloadRecord;

pub trait DownloadRepository: Send + Sync {
    fn create_download(&self, record: &DownloadRecord) -> Result<()>;
    fn get_download(&self, id: Uuid) -> Result<Option<DownloadRecord>>;
    fn list_active(&self) -> Result<Vec<DownloadRecord>>;
}
