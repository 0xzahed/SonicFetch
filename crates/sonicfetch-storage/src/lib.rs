pub mod models;
pub mod repository;

pub use models::{DownloadRecord, SegmentRecord};
pub use repository::SqliteDownloadRepository;
