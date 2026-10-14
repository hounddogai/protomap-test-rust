use super::*;

use acme_kit::text;

/// Where a strict delegation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDelegationLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictDelegationLedgerStatus {
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

impl std::fmt::Display for StrictDelegationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict delegation ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationLedger {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub key_id: String,
    pub cluster: String,
    pub status: StrictDelegationLedgerStatus,
}

impl StrictDelegationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDelegationLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDelegationLedgerStatus) -> bool {
        match self.status {
            StrictDelegationLedgerStatus::Draft => {
                match next {
                    StrictDelegationLedgerStatus::Pending | StrictDelegationLedgerStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDelegationLedgerStatus::Pending => {
                match next {
                    StrictDelegationLedgerStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDelegationLedgerStatus::Active => {
                match next {
                    StrictDelegationLedgerStatus::Suspended | StrictDelegationLedgerStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDelegationLedgerStatus::Suspended => {
                return next == StrictDelegationLedgerStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictDelegationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict delegation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> StrictDelegationLedger {
        StrictDelegationLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a strict delegation ledger in status draft.
pub fn new_strict_delegation_ledger(code: &str, name: &str) -> StrictDelegationLedger {
    StrictDelegationLedger {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        key_id: "".to_string(),
        cluster: "".to_string(),
        status: StrictDelegationLedgerStatus::Draft,
    }
}
