# Download State Machine

Date: 2026-10-30
Status: Canonical lifecycle. Normal SQLite state changes validate through `StateTransition`; startup recovery is the explicit repair exception.

## 1. Canonical States

- Queued
- Preparing
- Downloading
- Paused
- Verifying
- Completed
- Retrying
- Failed
- Cancelled
- NeedsAttention

## 2. Lifecycle Diagram

```mermaid
stateDiagram-v2
    [*] --> Queued
    Queued --> Preparing
    Queued --> Paused
    Queued --> Cancelled
    Preparing --> Downloading
    Preparing --> Paused
    Preparing --> Failed
    Preparing --> Cancelled
    Preparing --> NeedsAttention
    Downloading --> Paused
    Downloading --> Verifying
    Verifying --> Completed
    Downloading --> Failed
    Downloading --> Cancelled
    Downloading --> NeedsAttention
    Paused --> Cancelled
    Paused --> Preparing
    Failed --> Retrying
    Cancelled --> Retrying
    Retrying --> Cancelled
    Retrying --> Preparing
    Verifying --> Failed
    Verifying --> NeedsAttention
    NeedsAttention --> Queued
    NeedsAttention --> Cancelled
```

## 3. Pause/Resume Flow

```mermaid
flowchart TD
    A[Downloading] --> B[Pause requested]
    B --> C[Persist Paused state in SQLite]
    C --> D[Worker observes state or cancellation token]
    D --> E[Finish current bounded write and release process lease]
    E --> F[Partial bytes and validators remain on disk]
    F --> G[Paused]
    G --> H[Resume requested]
    H --> I[Claim worker lease and revalidate local metadata]
    I --> J{ETag or Last-Modified available?}
    J -->|Yes| K[Request remaining byte range with If-Range]
    J -->|No| L[Restart from byte zero]
    K --> M{Range and validator match?}
    M -->|Yes| N[Append remaining bytes]
    M -->|No| O[Discard partial and require retry from byte zero]
```

## 4. Error and Retry Flow

```mermaid
flowchart TD
    A[Failure while preparing, downloading, or finalizing] --> B[Persist Failed with safe diagnostic]
    B --> C[User invokes Retry]
    C --> D[Transition to Retrying]
    D --> E[Claim worker lease and prepare transfer]
    E --> F{Saved validator and partial still match?}
    F -->|Yes| G[Resume remaining bytes]
    F -->|No| H[Discard partial and restart safely]
    G --> I{Transfer and finalization succeed?}
    H --> I
    I -->|Yes| J[Completed]
    I -->|No| B
```

## 5. Transition Rules

Valid target transitions:
- Queued -> Preparing | Paused | Cancelled
- Preparing -> Downloading | Paused | Failed | Cancelled | NeedsAttention
- Downloading -> Paused | Verifying | Failed | Cancelled | NeedsAttention
- Paused -> Preparing | Cancelled | NeedsAttention
- Failed -> Retrying | Cancelled
- Cancelled -> Retrying
- Retrying -> Preparing | Cancelled | Failed
- Verifying -> Completed | Failed | NeedsAttention
- NeedsAttention -> Queued | Cancelled

Invalid transitions examples:
- Completed -> Downloading
- Cancelled -> Downloading
- Queued -> Completed
- Verifying -> Downloading

## 6. Persistence and recovery

```mermaid
flowchart TD
    A[Application opens SQLite] --> B[Apply schema and compatibility migration]
    B --> C[Inspect active states and worker_pid leases]
    C --> D{Lease process still exists?}
    D -->|Yes| E[Leave active task intact]
    D -->|No| F{Was state Verifying?}
    F -->|No| G[Restore task to Queued; clear stale lease]
    F -->|Yes| H{Final file published and temp absent or same inode?}
    H -->|Yes| I[Recover as Completed]
    H -->|No final exists but cannot verify| J[NeedsAttention]
    H -->|No final file| G
    E --> K[Expose current queue]
    G --> K
    I --> K
    J --> K
```

Required persisted events:

- State transitions with timestamp and reason.
- Retry attempt number and next-at timestamp.
- Validator changes (ETag/Last-Modified mismatch).
- Segment offsets and bytes written.

Completion guard:
- Mark Completed only after all of:
  - successful stream/segment merge
  - final fsync/rename finalize
  - optional checksum verification (if supplied)

## 7. Cancellation Semantics

- Cancellation is cooperative and token-driven.
- Active workers must stop fetch loops promptly.
- Partial progress must be committed before state change to Paused/Cancelled.

Current implementation note:
- The CLI prototype persists the worker PID and validates normal conditional SQLite state transitions. The worker checks control state while waiting on network reads, stores byte offsets and validators, and releases its lease on normal task completion. Download history and transactional audit entries are still unimplemented.
- Retries are user-triggered; automatic backoff, retry budgets, and persisted retry history are not implemented.

## 8. Required Unit Tests

- Valid transitions accepted.
- Invalid transitions rejected with typed errors.
- Cancellation during Downloading persists partial offsets.
- Retry path honors max attempts and backoff.
- Recovery rebuilds state from persisted metadata.
