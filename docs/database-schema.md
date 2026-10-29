# Database Schema (SQLite)

Date: 2026-10-30
Status: Initial normalized schema for SonicFetch persistence.

## 1. Design Goals

- Resume and recovery reliability.
- Explicit state history and diagnostics.
- Minimal sensitive-data storage.
- Efficient query paths for active/queued/completed views.

## 2. Tables

## downloads

Purpose:
- Primary download task record.

Columns:
- id TEXT PRIMARY KEY
- source_url TEXT NOT NULL
- sanitized_filename TEXT NOT NULL
- destination_dir TEXT NOT NULL
- temp_path TEXT NOT NULL
- final_path TEXT NOT NULL
- state TEXT NOT NULL
- priority INTEGER NOT NULL DEFAULT 0
- total_bytes INTEGER
- downloaded_bytes INTEGER NOT NULL DEFAULT 0
- etag TEXT
- last_modified TEXT
- checksum_algo TEXT
- checksum_expected TEXT
- supports_ranges INTEGER NOT NULL DEFAULT 0
- max_segments INTEGER NOT NULL DEFAULT 1
- retry_count INTEGER NOT NULL DEFAULT 0
- max_retries INTEGER NOT NULL DEFAULT 5
- next_retry_at TEXT
- error_code TEXT
- error_message TEXT
- category_id TEXT
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL
- started_at TEXT
- completed_at TEXT

Constraints:
- CHECK state IN ('Queued','Preparing','Downloading','Paused','Verifying','Completed','Retrying','Failed','Cancelled','NeedsAttention')
- CHECK max_segments >= 1
- CHECK retry_count >= 0
- FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL

## download_segments

Purpose:
- Segment metadata and progress for resumable segmented transfers.

Columns:
- id TEXT PRIMARY KEY
- download_id TEXT NOT NULL
- segment_index INTEGER NOT NULL
- range_start INTEGER NOT NULL
- range_end INTEGER NOT NULL
- bytes_written INTEGER NOT NULL DEFAULT 0
- state TEXT NOT NULL
- etag_at_creation TEXT
- last_modified_at_creation TEXT
- last_error_code TEXT
- updated_at TEXT NOT NULL

Constraints:
- UNIQUE(download_id, segment_index)
- CHECK range_start >= 0
- CHECK range_end >= range_start
- CHECK bytes_written >= 0
- FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE

## categories

Columns:
- id TEXT PRIMARY KEY
- name TEXT NOT NULL UNIQUE
- color TEXT
- created_at TEXT NOT NULL

## app_settings

Columns:
- key TEXT PRIMARY KEY
- value_json TEXT NOT NULL
- updated_at TEXT NOT NULL

Notes:
- Store JSON-encoded typed values validated in application layer.

## download_history

Purpose:
- Immutable event log for transitions and operational diagnostics.

Columns:
- id INTEGER PRIMARY KEY AUTOINCREMENT
- download_id TEXT NOT NULL
- event_type TEXT NOT NULL
- from_state TEXT
- to_state TEXT
- payload_json TEXT
- created_at TEXT NOT NULL

Constraints:
- FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE

## 3. Indexes

Required indexes:
- idx_downloads_state_priority_created ON downloads(state, priority DESC, created_at ASC)
- idx_downloads_updated_at ON downloads(updated_at DESC)
- idx_downloads_category ON downloads(category_id)
- idx_segments_download_id ON download_segments(download_id)
- idx_history_download_created ON download_history(download_id, created_at DESC)

## 4. Migration Strategy

- SQL migrations in migrations/ with monotonic numeric prefix.
- Startup migration check runs before engine startup.
- Failed migration aborts startup with explicit error.
- Never apply destructive schema changes without migration plan and backup notes.

## 5. Transaction Boundaries

Use transactions for:
- State transition + history append.
- Segment progress batch update + aggregate bytes update.
- Finalization transition (Verifying -> Completed/Failed) with integrity result.

## 6. Recovery Behavior

Startup recovery algorithm:
1. Load downloads in Preparing/Downloading/Retrying/Verifying.
2. Validate temp files and segment metadata consistency.
3. If consistent, move to Queued (or Paused depending policy).
4. If inconsistent/corrupt metadata, move to NeedsAttention with diagnostic history entry.

## 7. Sensitive Data Policy

Do not store:
- Authorization headers
- Cookies
- Session tokens
- Full signed URLs in logs/history payload

Storage redaction rule:
- source_url may be stored for functionality; log output must redact credentials and volatile signed query fields.
