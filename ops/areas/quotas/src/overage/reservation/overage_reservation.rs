use super::*;

use acme_kit::text;

/// Where a overage reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl OverageReservationStatus {
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

impl std::fmt::Display for OverageReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageReservation {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub notes: String,
    pub issuer: String,
    pub status: OverageReservationStatus,
}

impl OverageReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageReservationStatus) -> bool {
        match self.status {
            OverageReservationStatus::Open => {
                return next == OverageReservationStatus::Resolved;
            }
            OverageReservationStatus::InReview => {
                return next == OverageReservationStatus::Reopened || next == OverageReservationStatus::Archived;
            }
            OverageReservationStatus::Resolved => {
                return next == OverageReservationStatus::Reopened || next == OverageReservationStatus::Archived;
            }
            OverageReservationStatus::Reopened => {
                match next {
                    OverageReservationStatus::Archived => {
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

    pub fn move_to(&mut self, next: OverageReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> OverageReservation {
        OverageReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            notes: self.notes.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a overage reservation in status open.
pub fn new_overage_reservation(code: &str, title: &str) -> OverageReservation {
    OverageReservation {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        notes: "".to_string(),
        issuer: "".to_string(),
        status: OverageReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_overage_reservation("code-480", "title-90");
        assert_eq!(item.status, OverageReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_overage_reservation("code-480", "title-90");
        if item.move_to(OverageReservationStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageReservationStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_overage_reservation("code-480", "title-90");
        item.status = OverageReservationStatus::Archived;
        assert!(
            item.move_to(OverageReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
