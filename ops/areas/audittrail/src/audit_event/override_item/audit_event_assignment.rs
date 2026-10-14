use super::*;

use acme_kit::text;

/// Where a audit event assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAssignment {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub batch_size: i64,
    pub category: String,
    pub issuer: String,
    pub ceiling: f64,
    pub locale: String,
    pub status: AuditEventAssignmentStatus,
}

impl AuditEventAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventAssignmentStatus) -> bool {
        match self.status {
            AuditEventAssignmentStatus::Proposed => {
                return next == AuditEventAssignmentStatus::Approved;
            }
            AuditEventAssignmentStatus::Approved => {
                return next == AuditEventAssignmentStatus::Rejected || next == AuditEventAssignmentStatus::Withdrawn;
            }
            AuditEventAssignmentStatus::Rejected => {
                return next == AuditEventAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.batch_size as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventAssignment {
        AuditEventAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            batch_size: self.batch_size,
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a audit event assignment in status proposed.
pub fn new_audit_event_assignment(code: &str, name: &str) -> AuditEventAssignment {
    AuditEventAssignment {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        batch_size: 0,
        category: "".to_string(),
        issuer: "".to_string(),
        ceiling: 0.0,
        locale: "".to_string(),
        status: AuditEventAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_audit_event_assignment("code-348", "name-679");
        assert_eq!(item.status, AuditEventAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_audit_event_assignment("code-348", "name-679");
        if item.move_to(AuditEventAssignmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventAssignmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_audit_event_assignment("code-348", "name-679");
        item.status = AuditEventAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(AuditEventAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
