use super::*;

use acme_kit::text;

/// Where a approval checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ApprovalCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ApprovalCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One approval checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpoint {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub timezone: String,
    pub locale: String,
    pub audience: String,
    pub rotation_days: i64,
    pub budget: f64,
    pub status: ApprovalCheckpointStatus,
}

impl ApprovalCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalCheckpointStatus) -> bool {
        match self.status {
            ApprovalCheckpointStatus::Planned => {
                match next {
                    ApprovalCheckpointStatus::Scheduled | ApprovalCheckpointStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalCheckpointStatus::Scheduled => {
                return next == ApprovalCheckpointStatus::Completed || next == ApprovalCheckpointStatus::Cancelled;
            }
            ApprovalCheckpointStatus::InProgress => {
                return next == ApprovalCheckpointStatus::Cancelled;
            }
            ApprovalCheckpointStatus::Completed => {
                return next == ApprovalCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.limit as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalCheckpoint {
        ApprovalCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            timezone: self.timezone.clone(),
            locale: self.locale.clone(),
            audience: self.audience.clone(),
            rotation_days: self.rotation_days,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a approval checkpoint in status planned.
pub fn new_approval_checkpoint(code: &str, title: &str) -> ApprovalCheckpoint {
    ApprovalCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        timezone: "".to_string(),
        locale: "".to_string(),
        audience: "".to_string(),
        rotation_days: 0,
        budget: 0.0,
        status: ApprovalCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_approval_checkpoint("code-676", "title-902");
        assert_eq!(item.status, ApprovalCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_approval_checkpoint("code-676", "title-902");
        if item.move_to(ApprovalCheckpointStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalCheckpointStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_approval_checkpoint("code-676", "title-902");
        item.status = ApprovalCheckpointStatus::Cancelled;
        assert!(
            item.move_to(ApprovalCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
