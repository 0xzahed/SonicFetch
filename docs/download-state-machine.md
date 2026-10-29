# Download State Machine

Date: 2026-10-30
Status: Required lifecycle definition for SonicFetch engine.

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
    Preparing --> Downloading
    Downloading --> Paused
    Paused --> Downloading
    Downloading --> Verifying
    Verifying --> Completed

    Downloading --> Retrying
    Retrying --> Downloading
    Retrying --> Failed

    Downloading --> Cancelled
    Preparing --> Cancelled
    Paused --> Cancelled
    Retrying --> Cancelled

    Preparing --> NeedsAttention
    Downloading --> NeedsAttention
    Paused --> NeedsAttention
    NeedsAttention --> Queued
    NeedsAttention --> Cancelled
```

## 3. Pause/Resume Flow

```mermaid
flowchart TD
    A[Downloading] --> B[Pause requested]
    B --> C[CancellationToken trip for active workers]
    C --> D[Flush segment/file progress to storage]
    D --> E[Transition to Paused]
    E --> F[Resume requested]
    F --> G[Revalidate metadata and validators]
    G --> H{Resume still valid?}
    H -->|Yes| I[Transition to Downloading]
    H -->|No| J[Transition to NeedsAttention or restart path]
```

## 4. Error and Retry Flow

```mermaid
flowchart TD
    A[Failure in Preparing or Downloading] --> B{Transient?}
    B -->|Yes| C[RetryPolicy computes backoff]
    C --> D[Transition Retrying]
    D --> E{Attempts remaining?}
    E -->|Yes| F[Back to Downloading]
    E -->|No| G[Failed]
    B -->|No| G
```

## 5. Transition Rules

Valid transitions (subset):
- Queued -> Preparing
- Preparing -> Downloading | Cancelled | NeedsAttention
- Downloading -> Paused | Verifying | Retrying | Cancelled | NeedsAttention
- Paused -> Downloading | Cancelled | NeedsAttention
- Retrying -> Downloading | Failed | Cancelled
- Verifying -> Completed | Failed | NeedsAttention
- Failed -> Retrying | Cancelled
- NeedsAttention -> Queued | Cancelled

Invalid transitions examples:
- Completed -> Downloading
- Cancelled -> Downloading
- Queued -> Completed
- Verifying -> Downloading

## 6. Persistence Rules

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

## 8. Required Unit Tests

- Valid transitions accepted.
- Invalid transitions rejected with typed errors.
- Cancellation during Downloading persists partial offsets.
- Retry path honors max attempts and backoff.
- Recovery rebuilds state from persisted metadata.
