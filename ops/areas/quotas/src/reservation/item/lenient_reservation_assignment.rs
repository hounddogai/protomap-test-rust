use super::*;

use acme_kit::text;

/// Where a lenient reservation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientReservationAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientReservationAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientReservationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient reservation assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAssignment {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub archived: bool,
    pub ends_at: i64,
    pub priority: i64,
    pub status: LenientReservationAssignmentStatus,
}

impl LenientReservationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientReservationAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientReservationAssignmentStatus) -> bool {
        match self.status {
            LenientReservationAssignmentStatus::Proposed => {
                match next {
                    LenientReservationAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientReservationAssignmentStatus::Approved => {
                return next == LenientReservationAssignmentStatus::Withdrawn;
            }
            LenientReservationAssignmentStatus::Rejected => {
                return next == LenientReservationAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientReservationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient reservation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.ends_at as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientReservationAssignment {
        LenientReservationAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            archived: self.archived,
            ends_at: self.ends_at,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient reservation assignment in status proposed.
pub fn new_lenient_reservation_assignment(code: &str, name: &str) -> LenientReservationAssignment {
    LenientReservationAssignment {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        archived: false,
        ends_at: 0,
        priority: 0,
        status: LenientReservationAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_reservation_assignment("code-514", "name-946");
        assert_eq!(item.status, LenientReservationAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_lenient_reservation_assignment("code-514", "name-946");
        if item.move_to(LenientReservationAssignmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientReservationAssignmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_reservation_assignment("code-514", "name-946");
        item.status = LenientReservationAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(LenientReservationAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
