use super::*;

use acme_kit::text;

/// Where a strict audit event assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAuditEventAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictAuditEventAssignmentStatus {
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

impl std::fmt::Display for StrictAuditEventAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict audit event assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventAssignment {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub country: String,
    pub updated_at: i64,
    pub audience: String,
    pub segment: String,
    pub quantity: i64,
    pub created_at: i64,
    pub duration_minutes: i64,
    pub status: StrictAuditEventAssignmentStatus,
}

impl StrictAuditEventAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAuditEventAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAuditEventAssignmentStatus) -> bool {
        match self.status {
            StrictAuditEventAssignmentStatus::Draft => {
                return next == StrictAuditEventAssignmentStatus::Pending;
            }
            StrictAuditEventAssignmentStatus::Pending => {
                match next {
                    StrictAuditEventAssignmentStatus::Active | StrictAuditEventAssignmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAuditEventAssignmentStatus::Active => {
                match next {
                    StrictAuditEventAssignmentStatus::Suspended | StrictAuditEventAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAuditEventAssignmentStatus::Suspended => {
                return next == StrictAuditEventAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictAuditEventAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict audit event assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict audit event assignment", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictAuditEventAssignment {
        StrictAuditEventAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            country: self.country.clone(),
            updated_at: self.updated_at,
            audience: self.audience.clone(),
            segment: self.segment.clone(),
            quantity: self.quantity,
            created_at: self.created_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict audit event assignment in status draft.
pub fn new_strict_audit_event_assignment(code: &str, title: &str) -> StrictAuditEventAssignment {
    StrictAuditEventAssignment {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        country: "".to_string(),
        updated_at: 0,
        audience: "".to_string(),
        segment: "".to_string(),
        quantity: 0,
        created_at: 0,
        duration_minutes: 0,
        status: StrictAuditEventAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_audit_event_assignment("code-776", "title-718");
        assert_eq!(item.status, StrictAuditEventAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_strict_audit_event_assignment("code-776", "title-718");
        if item.move_to(StrictAuditEventAssignmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAuditEventAssignmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_audit_event_assignment("code-776", "title-718");
        item.status = StrictAuditEventAssignmentStatus::Closed;
        assert!(
            item.move_to(StrictAuditEventAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
