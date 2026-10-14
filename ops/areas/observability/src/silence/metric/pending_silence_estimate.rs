use super::*;

use acme_kit::text;

/// Where a pending silence estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSilenceEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingSilenceEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PendingSilenceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending silence estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceEstimate {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub namespace: String,
    pub category: String,
    pub status: PendingSilenceEstimateStatus,
}

impl PendingSilenceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSilenceEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSilenceEstimateStatus) -> bool {
        match self.status {
            PendingSilenceEstimateStatus::New => {
                match next {
                    PendingSilenceEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSilenceEstimateStatus::Queued => {
                match next {
                    PendingSilenceEstimateStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSilenceEstimateStatus::Running => {
                return next == PendingSilenceEstimateStatus::Done;
            }
            PendingSilenceEstimateStatus::Done => {
                return next == PendingSilenceEstimateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingSilenceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending silence estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending silence estimate", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> PendingSilenceEstimate {
        PendingSilenceEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            namespace: self.namespace.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending silence estimate in status new.
pub fn new_pending_silence_estimate(code: &str, title: &str) -> PendingSilenceEstimate {
    PendingSilenceEstimate {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        namespace: "".to_string(),
        category: "".to_string(),
        status: PendingSilenceEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_silence_estimate("code-424", "title-819");
        assert_eq!(item.status, PendingSilenceEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_pending_silence_estimate("code-424", "title-819");
        if item.move_to(PendingSilenceEstimateStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSilenceEstimateStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_silence_estimate("code-424", "title-819");
        item.status = PendingSilenceEstimateStatus::Failed;
        assert!(
            item.move_to(PendingSilenceEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
