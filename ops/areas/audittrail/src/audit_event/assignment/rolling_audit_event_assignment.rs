use super::*;

use acme_kit::text;

/// Where a rolling audit event assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingAuditEventAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingAuditEventAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingAuditEventAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling audit event assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventAssignment {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub quantity: i64,
    pub phone: String,
    pub priority: i64,
    pub status: RollingAuditEventAssignmentStatus,
}

impl RollingAuditEventAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingAuditEventAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingAuditEventAssignmentStatus) -> bool {
        match self.status {
            RollingAuditEventAssignmentStatus::Proposed => {
                return next == RollingAuditEventAssignmentStatus::Approved
                    || next == RollingAuditEventAssignmentStatus::Rejected;
            }
            RollingAuditEventAssignmentStatus::Approved => {
                match next {
                    RollingAuditEventAssignmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingAuditEventAssignmentStatus::Rejected => {
                match next {
                    RollingAuditEventAssignmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingAuditEventAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling audit event assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.priority as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingAuditEventAssignment {
        RollingAuditEventAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            quantity: self.quantity,
            phone: self.phone.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling audit event assignment in status proposed.
pub fn new_rolling_audit_event_assignment(code: &str, name: &str) -> RollingAuditEventAssignment {
    RollingAuditEventAssignment {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        quantity: 0,
        phone: "".to_string(),
        priority: 0,
        status: RollingAuditEventAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_audit_event_assignment("code-312", "name-486");
        assert_eq!(item.status, RollingAuditEventAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_audit_event_assignment("code-312", "name-486");
        if item.move_to(RollingAuditEventAssignmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingAuditEventAssignmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_audit_event_assignment("code-312", "name-486");
        item.status = RollingAuditEventAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(RollingAuditEventAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
