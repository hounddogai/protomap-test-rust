use super::*;

use acme_kit::text;

/// Where a pending approval checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingApprovalCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingApprovalCheckpointStatus {
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

impl std::fmt::Display for PendingApprovalCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending approval checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalCheckpoint {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub phone: String,
    pub tags: Vec<String>,
    pub minimum: f64,
    pub duration_minutes: i64,
    pub floor_value: f64,
    pub cost: f64,
    pub pinned: bool,
    pub status: PendingApprovalCheckpointStatus,
}

impl PendingApprovalCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingApprovalCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingApprovalCheckpointStatus) -> bool {
        match self.status {
            PendingApprovalCheckpointStatus::Planned => {
                return next == PendingApprovalCheckpointStatus::Scheduled
                    || next == PendingApprovalCheckpointStatus::Completed;
            }
            PendingApprovalCheckpointStatus::Scheduled => {
                match next {
                    PendingApprovalCheckpointStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingApprovalCheckpointStatus::InProgress => {
                return next == PendingApprovalCheckpointStatus::Completed;
            }
            PendingApprovalCheckpointStatus::Completed => {
                return next == PendingApprovalCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingApprovalCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending approval checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.threshold * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingApprovalCheckpoint {
        PendingApprovalCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            phone: self.phone.clone(),
            tags: self.tags.clone(),
            minimum: self.minimum,
            duration_minutes: self.duration_minutes,
            floor_value: self.floor_value,
            cost: self.cost,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending approval checkpoint in status planned.
pub fn new_pending_approval_checkpoint(code: &str, title: &str) -> PendingApprovalCheckpoint {
    PendingApprovalCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        phone: "".to_string(),
        tags: Vec::new(),
        minimum: 0.0,
        duration_minutes: 0,
        floor_value: 0.0,
        cost: 0.0,
        pinned: false,
        status: PendingApprovalCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_approval_checkpoint("code-981", "title-494");
        assert_eq!(item.status, PendingApprovalCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_pending_approval_checkpoint("code-981", "title-494");
        if item.move_to(PendingApprovalCheckpointStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingApprovalCheckpointStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_approval_checkpoint("code-981", "title-494");
        item.status = PendingApprovalCheckpointStatus::Cancelled;
        assert!(
            item.move_to(PendingApprovalCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
