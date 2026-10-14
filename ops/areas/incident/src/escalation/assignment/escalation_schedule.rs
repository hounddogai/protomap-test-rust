use super::*;

use acme_kit::text;

/// Where a escalation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EscalationScheduleStatus {
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

impl std::fmt::Display for EscalationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSchedule {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub budget: f64,
    pub currency: String,
    pub pinned: bool,
    pub rotation_days: i64,
    pub margin: f64,
    pub attempts: i64,
    pub port: i64,
    pub timezone: String,
    pub status: EscalationScheduleStatus,
}

impl EscalationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationScheduleStatus) -> bool {
        match self.status {
            EscalationScheduleStatus::Open => {
                return next == EscalationScheduleStatus::InReview || next == EscalationScheduleStatus::Reopened;
            }
            EscalationScheduleStatus::InReview => {
                return next == EscalationScheduleStatus::Archived;
            }
            EscalationScheduleStatus::Resolved => {
                match next {
                    EscalationScheduleStatus::Reopened | EscalationScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationScheduleStatus::Reopened => {
                return next == EscalationScheduleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation schedule", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.budget * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationSchedule {
        EscalationSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            budget: self.budget,
            currency: self.currency.clone(),
            pinned: self.pinned,
            rotation_days: self.rotation_days,
            margin: self.margin,
            attempts: self.attempts,
            port: self.port,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a escalation schedule in status open.
pub fn new_escalation_schedule(code: &str, title: &str) -> EscalationSchedule {
    EscalationSchedule {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        budget: 0.0,
        currency: "".to_string(),
        pinned: false,
        rotation_days: 0,
        margin: 0.0,
        attempts: 0,
        port: 0,
        timezone: "".to_string(),
        status: EscalationScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_escalation_schedule("code-121", "title-613");
        assert_eq!(item.status, EscalationScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_escalation_schedule("code-121", "title-613");
        if item.move_to(EscalationScheduleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationScheduleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_escalation_schedule("code-121", "title-613");
        item.status = EscalationScheduleStatus::Archived;
        assert!(
            item.move_to(EscalationScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
