use super::*;

use acme_kit::text;

/// Where a batched timeline policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTimelinePolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedTimelinePolicyStatus {
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

impl std::fmt::Display for BatchedTimelinePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched timeline policy as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelinePolicy {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub revision: i64,
    pub contact_email: String,
    pub status: BatchedTimelinePolicyStatus,
}

impl BatchedTimelinePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTimelinePolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTimelinePolicyStatus) -> bool {
        match self.status {
            BatchedTimelinePolicyStatus::Planned => {
                match next {
                    BatchedTimelinePolicyStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTimelinePolicyStatus::Scheduled => {
                match next {
                    BatchedTimelinePolicyStatus::InProgress | BatchedTimelinePolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTimelinePolicyStatus::InProgress => {
                match next {
                    BatchedTimelinePolicyStatus::Completed | BatchedTimelinePolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTimelinePolicyStatus::Completed => {
                match next {
                    BatchedTimelinePolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedTimelinePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched timeline policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> BatchedTimelinePolicy {
        BatchedTimelinePolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            revision: self.revision,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched timeline policy in status planned.
pub fn new_batched_timeline_policy(code: &str, title: &str) -> BatchedTimelinePolicy {
    BatchedTimelinePolicy {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        revision: 0,
        contact_email: "".to_string(),
        status: BatchedTimelinePolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_timeline_policy("code-252", "title-713");
        assert_eq!(item.status, BatchedTimelinePolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_batched_timeline_policy("code-252", "title-713");
        if item.move_to(BatchedTimelinePolicyStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTimelinePolicyStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_timeline_policy("code-252", "title-713");
        item.status = BatchedTimelinePolicyStatus::Cancelled;
        assert!(
            item.move_to(BatchedTimelinePolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
