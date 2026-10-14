use super::*;

use acme_kit::text;

/// Where a egress ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EgressLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EgressLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressLedger {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub owner_id: String,
    pub amount: f64,
    pub status: EgressLedgerStatus,
}

impl EgressLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressLedgerStatus) -> bool {
        match self.status {
            EgressLedgerStatus::Requested => {
                return next == EgressLedgerStatus::Expired;
            }
            EgressLedgerStatus::Granted => {
                match next {
                    EgressLedgerStatus::Revoked | EgressLedgerStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressLedgerStatus::Revoked => {
                return next == EgressLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.amount * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressLedger {
        EgressLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            owner_id: self.owner_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a egress ledger in status requested.
pub fn new_egress_ledger(code: &str, label: &str) -> EgressLedger {
    EgressLedger {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        owner_id: "".to_string(),
        amount: 0.0,
        status: EgressLedgerStatus::Requested,
    }
}
