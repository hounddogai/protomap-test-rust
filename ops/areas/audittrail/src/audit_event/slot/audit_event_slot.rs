use super::*;

use acme_kit::text;

/// Where a audit event slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AuditEventSlotStatus {
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

impl std::fmt::Display for AuditEventSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSlot {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub reviewer: String,
    pub cost: f64,
    pub budget: f64,
    pub created_at: i64,
    pub unit: String,
    pub rotation_days: i64,
    pub active: bool,
    pub status: AuditEventSlotStatus,
}

impl AuditEventSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventSlotStatus) -> bool {
        match self.status {
            AuditEventSlotStatus::Draft => {
                return next == AuditEventSlotStatus::Suspended;
            }
            AuditEventSlotStatus::Pending => {
                match next {
                    AuditEventSlotStatus::Active | AuditEventSlotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventSlotStatus::Active => {
                match next {
                    AuditEventSlotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventSlotStatus::Suspended => {
                match next {
                    AuditEventSlotStatus::Closed => {
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

    pub fn move_to(&mut self, next: AuditEventSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event slot {} cannot move from {} to {}",
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

    pub fn projected_exposure(&self) -> f64 {
        ((self.amount + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventSlot {
        AuditEventSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            reviewer: self.reviewer.clone(),
            cost: self.cost,
            budget: self.budget,
            created_at: self.created_at,
            unit: self.unit.clone(),
            rotation_days: self.rotation_days,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a audit event slot in status draft.
pub fn new_audit_event_slot(code: &str, name: &str) -> AuditEventSlot {
    AuditEventSlot {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        reviewer: "".to_string(),
        cost: 0.0,
        budget: 0.0,
        created_at: 0,
        unit: "".to_string(),
        rotation_days: 0,
        active: false,
        status: AuditEventSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_audit_event_slot("code-914", "name-192");
        assert_eq!(item.status, AuditEventSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_audit_event_slot("code-914", "name-192");
        if item.move_to(AuditEventSlotStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventSlotStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_audit_event_slot("code-914", "name-192");
        item.status = AuditEventSlotStatus::Closed;
        assert!(
            item.move_to(AuditEventSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
