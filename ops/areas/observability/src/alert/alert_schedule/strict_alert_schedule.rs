use super::*;

use acme_kit::text;

/// Where a strict alert schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAlertScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictAlertScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for StrictAlertScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict alert schedule as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertSchedule {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub phone: String,
    pub margin: f64,
    pub status: StrictAlertScheduleStatus,
}

impl StrictAlertSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAlertScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAlertScheduleStatus) -> bool {
        match self.status {
            StrictAlertScheduleStatus::Open => {
                return next == StrictAlertScheduleStatus::Resolved;
            }
            StrictAlertScheduleStatus::InReview => {
                return next == StrictAlertScheduleStatus::Archived;
            }
            StrictAlertScheduleStatus::Resolved => {
                match next {
                    StrictAlertScheduleStatus::Reopened | StrictAlertScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAlertScheduleStatus::Reopened => {
                return next == StrictAlertScheduleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictAlertScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict alert schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict alert schedule", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> StrictAlertSchedule {
        StrictAlertSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            phone: self.phone.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict alert schedule in status open.
pub fn new_strict_alert_schedule(code: &str, title: &str) -> StrictAlertSchedule {
    StrictAlertSchedule {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        phone: "".to_string(),
        margin: 0.0,
        status: StrictAlertScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_alert_schedule("code-292", "title-954");
        assert_eq!(item.status, StrictAlertScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_strict_alert_schedule("code-292", "title-954");
        if item.move_to(StrictAlertScheduleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAlertScheduleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_alert_schedule("code-292", "title-954");
        item.status = StrictAlertScheduleStatus::Archived;
        assert!(
            item.move_to(StrictAlertScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
