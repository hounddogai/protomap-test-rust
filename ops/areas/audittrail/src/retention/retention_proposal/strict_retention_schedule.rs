use super::*;

use acme_kit::text;

/// Where a strict retention schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRetentionScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictRetentionScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictRetentionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict retention schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionSchedule {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub margin: f64,
    pub minimum: f64,
    pub status: StrictRetentionScheduleStatus,
}

impl StrictRetentionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRetentionScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRetentionScheduleStatus) -> bool {
        match self.status {
            StrictRetentionScheduleStatus::Draft => {
                match next {
                    StrictRetentionScheduleStatus::Pending | StrictRetentionScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRetentionScheduleStatus::Pending => {
                return next == StrictRetentionScheduleStatus::Active;
            }
            StrictRetentionScheduleStatus::Active => {
                return next == StrictRetentionScheduleStatus::Suspended;
            }
            StrictRetentionScheduleStatus::Suspended => {
                return next == StrictRetentionScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictRetentionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict retention schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.margin - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRetentionSchedule {
        StrictRetentionSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            margin: self.margin,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict retention schedule in status draft.
pub fn new_strict_retention_schedule(code: &str, label: &str) -> StrictRetentionSchedule {
    StrictRetentionSchedule {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        margin: 0.0,
        minimum: 0.0,
        status: StrictRetentionScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_retention_schedule("code-635", "label-324");
        assert_eq!(item.status, StrictRetentionScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_strict_retention_schedule("code-635", "label-324");
        if item.move_to(StrictRetentionScheduleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRetentionScheduleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_retention_schedule("code-635", "label-324");
        item.status = StrictRetentionScheduleStatus::Closed;
        assert!(
            item.move_to(StrictRetentionScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
