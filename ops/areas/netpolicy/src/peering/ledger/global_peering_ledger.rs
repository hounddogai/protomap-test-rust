use super::*;

use acme_kit::text;

/// Where a global peering ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPeeringLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalPeeringLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalPeeringLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global peering ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringLedger {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub label: String,
    pub currency: String,
    pub quantity: i64,
    pub status: GlobalPeeringLedgerStatus,
}

impl GlobalPeeringLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPeeringLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPeeringLedgerStatus) -> bool {
        match self.status {
            GlobalPeeringLedgerStatus::Planned => {
                match next {
                    GlobalPeeringLedgerStatus::InProgress | GlobalPeeringLedgerStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPeeringLedgerStatus::Scheduled => {
                match next {
                    GlobalPeeringLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPeeringLedgerStatus::InProgress => {
                return next == GlobalPeeringLedgerStatus::Completed || next == GlobalPeeringLedgerStatus::Cancelled;
            }
            GlobalPeeringLedgerStatus::Completed => {
                return next == GlobalPeeringLedgerStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPeeringLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global peering ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> GlobalPeeringLedger {
        GlobalPeeringLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global peering ledger in status planned.
pub fn new_global_peering_ledger(code: &str, name: &str) -> GlobalPeeringLedger {
    GlobalPeeringLedger {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        label: "".to_string(),
        currency: "".to_string(),
        quantity: 0,
        status: GlobalPeeringLedgerStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_peering_ledger("code-253", "name-507");
        assert_eq!(item.status, GlobalPeeringLedgerStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_global_peering_ledger("code-253", "name-507");
        if item.move_to(GlobalPeeringLedgerStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPeeringLedgerStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_peering_ledger("code-253", "name-507");
        item.status = GlobalPeeringLedgerStatus::Cancelled;
        assert!(
            item.move_to(GlobalPeeringLedgerStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
