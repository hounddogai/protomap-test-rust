use super::*;

use acme_kit::text;

/// Where a policy reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PolicyReservationStatus {
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

impl std::fmt::Display for PolicyReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReservation {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub minimum: f64,
    pub currency: String,
    pub visible: bool,
    pub maximum: f64,
    pub port: i64,
    pub retries: i64,
    pub segment: String,
    pub status: PolicyReservationStatus,
}

impl PolicyReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyReservationStatus) -> bool {
        match self.status {
            PolicyReservationStatus::Planned => {
                return next == PolicyReservationStatus::Scheduled || next == PolicyReservationStatus::Completed;
            }
            PolicyReservationStatus::Scheduled => {
                match next {
                    PolicyReservationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyReservationStatus::InProgress => {
                match next {
                    PolicyReservationStatus::Completed | PolicyReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyReservationStatus::Completed => {
                return next == PolicyReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.grace_days as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyReservation {
        PolicyReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            minimum: self.minimum,
            currency: self.currency.clone(),
            visible: self.visible,
            maximum: self.maximum,
            port: self.port,
            retries: self.retries,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a policy reservation in status planned.
pub fn new_policy_reservation(code: &str, title: &str) -> PolicyReservation {
    PolicyReservation {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        minimum: 0.0,
        currency: "".to_string(),
        visible: false,
        maximum: 0.0,
        port: 0,
        retries: 0,
        segment: "".to_string(),
        status: PolicyReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_policy_reservation("code-713", "title-242");
        assert_eq!(item.status, PolicyReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_policy_reservation("code-713", "title-242");
        if item.move_to(PolicyReservationStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyReservationStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_policy_reservation("code-713", "title-242");
        item.status = PolicyReservationStatus::Cancelled;
        assert!(
            item.move_to(PolicyReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
