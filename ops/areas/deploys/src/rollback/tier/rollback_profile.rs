use super::*;

use acme_kit::text;

/// Where a rollback profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollbackProfileStatus {
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

impl std::fmt::Display for RollbackProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProfile {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub cipher: String,
    pub quota_units: i64,
    pub cluster: String,
    pub threshold: f64,
    pub archived: bool,
    pub segment: String,
    pub grace_days: i64,
    pub status: RollbackProfileStatus,
}

impl RollbackProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackProfileStatus) -> bool {
        match self.status {
            RollbackProfileStatus::New => {
                return next == RollbackProfileStatus::Running || next == RollbackProfileStatus::Done;
            }
            RollbackProfileStatus::Queued => {
                return next == RollbackProfileStatus::Running || next == RollbackProfileStatus::Done;
            }
            RollbackProfileStatus::Running => {
                return next == RollbackProfileStatus::Failed;
            }
            RollbackProfileStatus::Done => {
                match next {
                    RollbackProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback profile", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quota_units as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackProfile {
        RollbackProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            cipher: self.cipher.clone(),
            quota_units: self.quota_units,
            cluster: self.cluster.clone(),
            threshold: self.threshold,
            archived: self.archived,
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rollback profile in status new.
pub fn new_rollback_profile(code: &str, label: &str) -> RollbackProfile {
    RollbackProfile {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        cipher: "".to_string(),
        quota_units: 0,
        cluster: "".to_string(),
        threshold: 0.0,
        archived: false,
        segment: "".to_string(),
        grace_days: 0,
        status: RollbackProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollback_profile("code-890", "label-687");
        assert_eq!(item.status, RollbackProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rollback_profile("code-890", "label-687");
        if item.move_to(RollbackProfileStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackProfileStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollback_profile("code-890", "label-687");
        item.status = RollbackProfileStatus::Failed;
        assert!(
            item.move_to(RollbackProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
