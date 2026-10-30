PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS categories (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS downloads (
  id TEXT PRIMARY KEY,
  source_url TEXT NOT NULL,
  sanitized_filename TEXT NOT NULL,
  destination_dir TEXT NOT NULL,
  temp_path TEXT NOT NULL,
  final_path TEXT NOT NULL,
  state TEXT NOT NULL,
  priority INTEGER NOT NULL DEFAULT 0,
  total_bytes INTEGER,
  downloaded_bytes INTEGER NOT NULL DEFAULT 0,
  etag TEXT,
  last_modified TEXT,
  checksum_algo TEXT,
  checksum_expected TEXT,
  supports_ranges INTEGER NOT NULL DEFAULT 0,
  max_segments INTEGER NOT NULL DEFAULT 1,
  retry_count INTEGER NOT NULL DEFAULT 0,
  max_retries INTEGER NOT NULL DEFAULT 5,
  next_retry_at TEXT,
  error_code TEXT,
  error_message TEXT,
  worker_pid INTEGER,
  category_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  started_at TEXT,
  completed_at TEXT,
  CHECK (state IN ('Queued','Preparing','Downloading','Paused','Verifying','Completed','Retrying','Failed','Cancelled','NeedsAttention')),
  CHECK (max_segments >= 1),
  CHECK (retry_count >= 0),
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS download_segments (
  id TEXT PRIMARY KEY,
  download_id TEXT NOT NULL,
  segment_index INTEGER NOT NULL,
  range_start INTEGER NOT NULL,
  range_end INTEGER NOT NULL,
  bytes_written INTEGER NOT NULL DEFAULT 0,
  state TEXT NOT NULL,
  etag_at_creation TEXT,
  last_modified_at_creation TEXT,
  last_error_code TEXT,
  updated_at TEXT NOT NULL,
  UNIQUE(download_id, segment_index),
  CHECK (range_start >= 0),
  CHECK (range_end >= range_start),
  CHECK (bytes_written >= 0),
  FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS download_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  download_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  from_state TEXT,
  to_state TEXT,
  payload_json TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_downloads_state_priority_created
  ON downloads(state, priority DESC, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_downloads_updated_at
  ON downloads(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_downloads_category
  ON downloads(category_id);

CREATE INDEX IF NOT EXISTS idx_segments_download_id
  ON download_segments(download_id);

CREATE INDEX IF NOT EXISTS idx_history_download_created
  ON download_history(download_id, created_at DESC);
