use sonicfetch_core::DownloadState;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum DownloadCommand {
    Add { id: Uuid, url: String },
    Pause { id: Uuid },
    Resume { id: Uuid },
    Cancel { id: Uuid },
    Retry { id: Uuid },
    Remove { id: Uuid },
}

#[derive(Debug, Clone)]
pub struct DownloadEvent {
    pub id: Uuid,
    pub state: DownloadState,
    pub at: chrono::DateTime<chrono::Utc>,
}

pub struct DownloadManager {
    events: broadcast::Sender<DownloadEvent>,
}

impl DownloadManager {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(1024);
        Self { events }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DownloadEvent> {
        self.events.subscribe()
    }

    pub fn emit_state(&self, id: Uuid, state: DownloadState) {
        let _ = self.events.send(DownloadEvent {
            id,
            state,
            at: chrono::Utc::now(),
        });
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::DownloadManager;
    use sonicfetch_core::DownloadState;
    use uuid::Uuid;

    #[test]
    fn emits_state_event() {
        let manager = DownloadManager::new();
        let mut rx = manager.subscribe();
        let id = Uuid::new_v4();

        manager.emit_state(id, DownloadState::Queued);

        let ev = rx.try_recv().expect("event should be available");
        assert_eq!(ev.id, id);
        assert_eq!(ev.state, DownloadState::Queued);
    }
}
