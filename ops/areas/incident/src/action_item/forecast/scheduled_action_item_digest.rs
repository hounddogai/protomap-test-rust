use super::*;

use acme_kit::text;

/// Where a scheduled action item digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledActionItemDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledActionItemDigestStatus {
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

impl std::fmt::Display for ScheduledActionItemDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled action item digest as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemDigest {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub margin: f64,
    pub ratio: f64,
    pub status: ScheduledActionItemDigestStatus,
}

impl ScheduledActionItemDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledActionItemDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledActionItemDigestStatus) -> bool {
        match self.status {
            ScheduledActionItemDigestStatus::Planned => {
                return next == ScheduledActionItemDigestStatus::InProgress
                    || next == ScheduledActionItemDigestStatus::Completed;
            }
            ScheduledActionItemDigestStatus::Scheduled => {
                return next == ScheduledActionItemDigestStatus::InProgress
                    || next == ScheduledActionItemDigestStatus::Completed;
            }
            ScheduledActionItemDigestStatus::InProgress => {
                match next {
                    ScheduledActionItemDigestStatus::Completed | ScheduledActionItemDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledActionItemDigestStatus::Completed => {
                return next == ScheduledActionItemDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledActionItemDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled action item digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled action item digest", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.margin * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledActionItemDigest {
        ScheduledActionItemDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            margin: self.margin,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled action item digest in status planned.
pub fn new_scheduled_action_item_digest(code: &str, label: &str) -> ScheduledActionItemDigest {
    ScheduledActionItemDigest {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        margin: 0.0,
        ratio: 0.0,
        status: ScheduledActionItemDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_action_item_digest("code-350", "label-216");
        assert_eq!(item.status, ScheduledActionItemDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_scheduled_action_item_digest("code-350", "label-216");
        if item.move_to(ScheduledActionItemDigestStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledActionItemDigestStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_action_item_digest("code-350", "label-216");
        item.status = ScheduledActionItemDigestStatus::Cancelled;
        assert!(
            item.move_to(ScheduledActionItemDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
