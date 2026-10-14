use super::*;

use acme_kit::text;

/// Where a scheduled audit event allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledAuditEventAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledAuditEventAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledAuditEventAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled audit event allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventAllocation {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub cost: f64,
    pub archived: bool,
    pub status: ScheduledAuditEventAllocationStatus,
}

impl ScheduledAuditEventAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledAuditEventAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledAuditEventAllocationStatus) -> bool {
        match self.status {
            ScheduledAuditEventAllocationStatus::Requested => {
                match next {
                    ScheduledAuditEventAllocationStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledAuditEventAllocationStatus::Granted => {
                return next == ScheduledAuditEventAllocationStatus::Revoked
                    || next == ScheduledAuditEventAllocationStatus::Expired;
            }
            ScheduledAuditEventAllocationStatus::Revoked => {
                return next == ScheduledAuditEventAllocationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledAuditEventAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled audit event allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled audit event allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> ScheduledAuditEventAllocation {
        ScheduledAuditEventAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            cost: self.cost,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled audit event allocation in status requested.
pub fn new_scheduled_audit_event_allocation(code: &str, name: &str) -> ScheduledAuditEventAllocation {
    ScheduledAuditEventAllocation {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        cost: 0.0,
        archived: false,
        status: ScheduledAuditEventAllocationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_audit_event_allocation("code-191", "name-199");
        assert_eq!(item.status, ScheduledAuditEventAllocationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_scheduled_audit_event_allocation("code-191", "name-199");
        if item.move_to(ScheduledAuditEventAllocationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledAuditEventAllocationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_audit_event_allocation("code-191", "name-199");
        item.status = ScheduledAuditEventAllocationStatus::Expired;
        assert!(
            item.move_to(ScheduledAuditEventAllocationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
