use super::*;

use acme_kit::text;

/// Where a strict binding assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictBindingAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictBindingAssignmentStatus {
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

impl std::fmt::Display for StrictBindingAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict binding assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingAssignment {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub contact_email: String,
    pub timezone: String,
    pub cost: f64,
    pub fingerprint: String,
    pub status: StrictBindingAssignmentStatus,
}

impl StrictBindingAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictBindingAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictBindingAssignmentStatus) -> bool {
        match self.status {
            StrictBindingAssignmentStatus::Draft => {
                match next {
                    StrictBindingAssignmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictBindingAssignmentStatus::Pending => {
                return next == StrictBindingAssignmentStatus::Closed;
            }
            StrictBindingAssignmentStatus::Active => {
                return next == StrictBindingAssignmentStatus::Suspended;
            }
            StrictBindingAssignmentStatus::Suspended => {
                return next == StrictBindingAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictBindingAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict binding assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> StrictBindingAssignment {
        StrictBindingAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            contact_email: self.contact_email.clone(),
            timezone: self.timezone.clone(),
            cost: self.cost,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict binding assignment in status draft.
pub fn new_strict_binding_assignment(code: &str, label: &str) -> StrictBindingAssignment {
    StrictBindingAssignment {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        contact_email: "".to_string(),
        timezone: "".to_string(),
        cost: 0.0,
        fingerprint: "".to_string(),
        status: StrictBindingAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_binding_assignment("code-433", "label-281");
        assert_eq!(item.status, StrictBindingAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_strict_binding_assignment("code-433", "label-281");
        if item.move_to(StrictBindingAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictBindingAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_binding_assignment("code-433", "label-281");
        item.status = StrictBindingAssignmentStatus::Closed;
        assert!(
            item.move_to(StrictBindingAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
