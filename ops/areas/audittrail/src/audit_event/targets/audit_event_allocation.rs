use super::*;

use acme_kit::text;

/// Where a audit event allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AuditEventAllocationStatus {
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

impl std::fmt::Display for AuditEventAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event allocation as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAllocation {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub weight: f64,
    pub name: String,
    pub currency: String,
    pub reviewer: String,
    pub status: AuditEventAllocationStatus,
}

impl AuditEventAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventAllocationStatus) -> bool {
        match self.status {
            AuditEventAllocationStatus::Draft => {
                match next {
                    AuditEventAllocationStatus::Suspended | AuditEventAllocationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventAllocationStatus::Pending => {
                match next {
                    AuditEventAllocationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventAllocationStatus::Active => {
                match next {
                    AuditEventAllocationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventAllocationStatus::Suspended => {
                match next {
                    AuditEventAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: AuditEventAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.weight + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventAllocation {
        AuditEventAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            weight: self.weight,
            name: self.name.clone(),
            currency: self.currency.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a audit event allocation in status draft.
pub fn new_audit_event_allocation(code: &str, title: &str) -> AuditEventAllocation {
    AuditEventAllocation {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        weight: 0.0,
        name: "".to_string(),
        currency: "".to_string(),
        reviewer: "".to_string(),
        status: AuditEventAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_audit_event_allocation("code-371", "title-27");
        assert_eq!(item.status, AuditEventAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_audit_event_allocation("code-371", "title-27");
        if item.move_to(AuditEventAllocationStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventAllocationStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_audit_event_allocation("code-371", "title-27");
        item.status = AuditEventAllocationStatus::Closed;
        assert!(
            item.move_to(AuditEventAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
