use super::*;

use acme_kit::text;

/// Where a tiered health check reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredHealthCheckReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredHealthCheckReservationStatus {
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

impl std::fmt::Display for TieredHealthCheckReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered health check reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckReservation {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub currency: String,
    pub contact_email: String,
    pub status: TieredHealthCheckReservationStatus,
}

impl TieredHealthCheckReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredHealthCheckReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredHealthCheckReservationStatus) -> bool {
        match self.status {
            TieredHealthCheckReservationStatus::Open => {
                return next == TieredHealthCheckReservationStatus::InReview
                    || next == TieredHealthCheckReservationStatus::Archived;
            }
            TieredHealthCheckReservationStatus::InReview => {
                match next {
                    TieredHealthCheckReservationStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredHealthCheckReservationStatus::Resolved => {
                match next {
                    TieredHealthCheckReservationStatus::Reopened | TieredHealthCheckReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredHealthCheckReservationStatus::Reopened => {
                match next {
                    TieredHealthCheckReservationStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredHealthCheckReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered health check reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> TieredHealthCheckReservation {
        TieredHealthCheckReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered health check reservation in status open.
pub fn new_tiered_health_check_reservation(code: &str, label: &str) -> TieredHealthCheckReservation {
    TieredHealthCheckReservation {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        currency: "".to_string(),
        contact_email: "".to_string(),
        status: TieredHealthCheckReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_health_check_reservation("code-449", "label-146");
        assert_eq!(item.status, TieredHealthCheckReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_tiered_health_check_reservation("code-449", "label-146");
        if item.move_to(TieredHealthCheckReservationStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredHealthCheckReservationStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_health_check_reservation("code-449", "label-146");
        item.status = TieredHealthCheckReservationStatus::Archived;
        assert!(
            item.move_to(TieredHealthCheckReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
