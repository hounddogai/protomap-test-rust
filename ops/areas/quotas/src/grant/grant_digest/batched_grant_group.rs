use super::*;

use acme_kit::text;

/// Where a batched grant group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedGrantGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedGrantGroupStatus {
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

impl std::fmt::Display for BatchedGrantGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched grant group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantGroup {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub quota_units: i64,
    pub revision: i64,
    pub status: BatchedGrantGroupStatus,
}

impl BatchedGrantGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedGrantGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedGrantGroupStatus) -> bool {
        match self.status {
            BatchedGrantGroupStatus::Planned => {
                match next {
                    BatchedGrantGroupStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedGrantGroupStatus::Scheduled => {
                return next == BatchedGrantGroupStatus::Cancelled;
            }
            BatchedGrantGroupStatus::InProgress => {
                return next == BatchedGrantGroupStatus::Completed;
            }
            BatchedGrantGroupStatus::Completed => {
                return next == BatchedGrantGroupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedGrantGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched grant group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedGrantGroup {
        BatchedGrantGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            quota_units: self.quota_units,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched grant group in status planned.
pub fn new_batched_grant_group(code: &str, label: &str) -> BatchedGrantGroup {
    BatchedGrantGroup {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        quota_units: 0,
        revision: 0,
        status: BatchedGrantGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_grant_group("code-224", "label-536");
        assert_eq!(item.status, BatchedGrantGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_batched_grant_group("code-224", "label-536");
        if item.move_to(BatchedGrantGroupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedGrantGroupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_grant_group("code-224", "label-536");
        item.status = BatchedGrantGroupStatus::Cancelled;
        assert!(
            item.move_to(BatchedGrantGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
