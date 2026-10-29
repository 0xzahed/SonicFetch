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
                | (DownloadState::Preparing, DownloadState::Downloading)
                | (DownloadState::Preparing, DownloadState::Cancelled)
                | (DownloadState::Preparing, DownloadState::NeedsAttention)
                | (DownloadState::Downloading, DownloadState::Paused)
                | (DownloadState::Downloading, DownloadState::Verifying)
                | (DownloadState::Downloading, DownloadState::Retrying)
                | (DownloadState::Downloading, DownloadState::Cancelled)
                | (DownloadState::Downloading, DownloadState::NeedsAttention)
                | (DownloadState::Paused, DownloadState::Downloading)
                | (DownloadState::Paused, DownloadState::Cancelled)
                | (DownloadState::Paused, DownloadState::NeedsAttention)
                | (DownloadState::Retrying, DownloadState::Downloading)
                | (DownloadState::Retrying, DownloadState::Failed)
                | (DownloadState::Retrying, DownloadState::Cancelled)
                | (DownloadState::Verifying, DownloadState::Completed)
                | (DownloadState::Verifying, DownloadState::Failed)
                | (DownloadState::Verifying, DownloadState::NeedsAttention)
                | (DownloadState::Failed, DownloadState::Retrying)
                | (DownloadState::Failed, DownloadState::Cancelled)
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
}
