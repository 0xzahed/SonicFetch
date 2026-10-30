use crate::errors::LifecycleError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DownloadState {
    Queued,
    Preparing,
    Downloading,
    Paused,
    Verifying,
    Completed,
    Retrying,
    Failed,
    Cancelled,
    NeedsAttention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateTransition {
    pub from: DownloadState,
    pub to: DownloadState,
}

impl StateTransition {
    pub fn validate(self) -> Result<(), LifecycleError> {
        let allowed = matches!(
            (self.from, self.to),
            (DownloadState::Queued, DownloadState::Preparing)
                | (DownloadState::Queued, DownloadState::Paused)
                | (DownloadState::Queued, DownloadState::Cancelled)
                | (DownloadState::Preparing, DownloadState::Downloading)
                | (DownloadState::Preparing, DownloadState::Paused)
                | (DownloadState::Preparing, DownloadState::Failed)
                | (DownloadState::Preparing, DownloadState::Cancelled)
                | (DownloadState::Preparing, DownloadState::NeedsAttention)
                | (DownloadState::Downloading, DownloadState::Paused)
                | (DownloadState::Downloading, DownloadState::Verifying)
                | (DownloadState::Downloading, DownloadState::Failed)
                | (DownloadState::Downloading, DownloadState::Cancelled)
                | (DownloadState::Downloading, DownloadState::NeedsAttention)
                | (DownloadState::Paused, DownloadState::Preparing)
                | (DownloadState::Paused, DownloadState::Cancelled)
                | (DownloadState::Paused, DownloadState::NeedsAttention)
                | (DownloadState::Retrying, DownloadState::Preparing)
                | (DownloadState::Retrying, DownloadState::Failed)
                | (DownloadState::Retrying, DownloadState::Cancelled)
                | (DownloadState::Verifying, DownloadState::Completed)
                | (DownloadState::Verifying, DownloadState::Failed)
                | (DownloadState::Verifying, DownloadState::NeedsAttention)
                | (DownloadState::Failed, DownloadState::Retrying)
                | (DownloadState::Failed, DownloadState::Cancelled)
                | (DownloadState::Cancelled, DownloadState::Retrying)
                | (DownloadState::NeedsAttention, DownloadState::Queued)
                | (DownloadState::NeedsAttention, DownloadState::Cancelled)
        );

        if allowed {
            Ok(())
        } else {
            Err(LifecycleError {
                from: self.from,
                to: self.to,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DownloadState, StateTransition};

    #[test]
    fn allows_valid_transition() {
        let transition = StateTransition {
            from: DownloadState::Queued,
            to: DownloadState::Preparing,
        };

        assert!(transition.validate().is_ok());
    }

    #[test]
    fn rejects_invalid_transition() {
        let transition = StateTransition {
            from: DownloadState::Completed,
            to: DownloadState::Downloading,
        };

        assert!(transition.validate().is_err());
    }

    #[test]
    fn cancel_is_cooperative_from_retrying() {
        let transition = StateTransition {
            from: DownloadState::Retrying,
            to: DownloadState::Cancelled,
        };

        assert!(transition.validate().is_ok());
    }

    #[test]
    fn accepts_the_documented_lifecycle_edges() {
        use DownloadState::*;
        let allowed = [
            (Queued, Preparing), (Queued, Paused), (Queued, Cancelled),
            (Preparing, Downloading), (Preparing, Paused), (Preparing, Failed),
            (Preparing, Cancelled), (Preparing, NeedsAttention),
            (Downloading, Paused), (Downloading, Verifying), (Downloading, Failed),
            (Downloading, Cancelled), (Downloading, NeedsAttention),
            (Paused, Preparing), (Paused, Cancelled), (Paused, NeedsAttention),
            (Retrying, Preparing), (Retrying, Failed), (Retrying, Cancelled),
            (Verifying, Completed), (Verifying, Failed), (Verifying, NeedsAttention),
            (Failed, Retrying), (Failed, Cancelled), (Cancelled, Retrying),
            (NeedsAttention, Queued), (NeedsAttention, Cancelled),
        ];

        for (from, to) in allowed {
            assert!(StateTransition { from, to }.validate().is_ok(), "{from:?} -> {to:?}");
        }
    }

    #[test]
    fn rejects_terminal_and_unsupported_edges() {
        use DownloadState::*;
        let rejected = [
            (Completed, Downloading),
            (Completed, Failed),
            (Cancelled, Downloading),
            (Queued, Completed),
            (Verifying, Paused),
            (Downloading, Preparing),
        ];

        for (from, to) in rejected {
            assert!(StateTransition { from, to }.validate().is_err(), "{from:?} -> {to:?}");
        }
    }
}
