use super::*;

use acme_kit::text;

/// Where a tiered reservation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredReservationAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredReservationAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredReservationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered reservation assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationAssignment {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub title: String,
    pub fingerprint: String,
    pub timezone: String,
    pub status: TieredReservationAssignmentStatus,
}

impl TieredReservationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredReservationAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredReservationAssignmentStatus) -> bool {
        match self.status {
            TieredReservationAssignmentStatus::Created => {
                return next == TieredReservationAssignmentStatus::Published;
            }
            TieredReservationAssignmentStatus::Validated => {
                match next {
                    TieredReservationAssignmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredReservationAssignmentStatus::Published => {
                match next {
                    TieredReservationAssignmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredReservationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered reservation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered reservation assignment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> TieredReservationAssignment {
        TieredReservationAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered reservation assignment in status created.
pub fn new_tiered_reservation_assignment(code: &str, label: &str) -> TieredReservationAssignment {
    TieredReservationAssignment {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        title: "".to_string(),
        fingerprint: "".to_string(),
        timezone: "".to_string(),
        status: TieredReservationAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_reservation_assignment("code-995", "label-586");
        assert_eq!(item.status, TieredReservationAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_tiered_reservation_assignment("code-995", "label-586");
        if item.move_to(TieredReservationAssignmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredReservationAssignmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_reservation_assignment("code-995", "label-586");
        item.status = TieredReservationAssignmentStatus::Retired;
        assert!(
            item.move_to(TieredReservationAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
