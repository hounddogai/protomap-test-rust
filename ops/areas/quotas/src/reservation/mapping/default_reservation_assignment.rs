use super::*;

use acme_kit::text;

/// Where a default reservation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReservationAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultReservationAssignmentStatus {
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

impl std::fmt::Display for DefaultReservationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default reservation assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignment {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub archived: bool,
    pub principal: String,
    pub status: DefaultReservationAssignmentStatus,
}

impl DefaultReservationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReservationAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReservationAssignmentStatus) -> bool {
        match self.status {
            DefaultReservationAssignmentStatus::Draft => {
                return next == DefaultReservationAssignmentStatus::Active
                    || next == DefaultReservationAssignmentStatus::Closed;
            }
            DefaultReservationAssignmentStatus::Pending => {
                return next == DefaultReservationAssignmentStatus::Active
                    || next == DefaultReservationAssignmentStatus::Closed;
            }
            DefaultReservationAssignmentStatus::Active => {
                return next == DefaultReservationAssignmentStatus::Suspended;
            }
            DefaultReservationAssignmentStatus::Suspended => {
                return next == DefaultReservationAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultReservationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default reservation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> DefaultReservationAssignment {
        DefaultReservationAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            archived: self.archived,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default reservation assignment in status draft.
pub fn new_default_reservation_assignment(code: &str, name: &str) -> DefaultReservationAssignment {
    DefaultReservationAssignment {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        archived: false,
        principal: "".to_string(),
        status: DefaultReservationAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_reservation_assignment("code-321", "name-693");
        assert_eq!(item.status, DefaultReservationAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_default_reservation_assignment("code-321", "name-693");
        if item.move_to(DefaultReservationAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultReservationAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_reservation_assignment("code-321", "name-693");
        item.status = DefaultReservationAssignmentStatus::Closed;
        assert!(
            item.move_to(DefaultReservationAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
