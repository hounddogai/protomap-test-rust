use super::*;

use acme_kit::text;

/// Where a approval target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ApprovalTargetStatus {
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

impl std::fmt::Display for ApprovalTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval target as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTarget {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub audience: String,
    pub unit: String,
    pub notes: String,
    pub grace_days: i64,
    pub status: ApprovalTargetStatus,
}

impl ApprovalTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalTargetStatus) -> bool {
        match self.status {
            ApprovalTargetStatus::New => {
                return next == ApprovalTargetStatus::Queued || next == ApprovalTargetStatus::Failed;
            }
            ApprovalTargetStatus::Queued => {
                return next == ApprovalTargetStatus::Done;
            }
            ApprovalTargetStatus::Running => {
                return next == ApprovalTargetStatus::Done || next == ApprovalTargetStatus::Failed;
            }
            ApprovalTargetStatus::Done => {
                return next == ApprovalTargetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "approval target", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.expires_at as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalTarget {
        ApprovalTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            audience: self.audience.clone(),
            unit: self.unit.clone(),
            notes: self.notes.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a approval target in status new.
pub fn new_approval_target(code: &str, label: &str) -> ApprovalTarget {
    ApprovalTarget {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        audience: "".to_string(),
        unit: "".to_string(),
        notes: "".to_string(),
        grace_days: 0,
        status: ApprovalTargetStatus::New,
    }
}
