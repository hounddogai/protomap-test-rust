use super::*;

use acme_kit::text;

/// Where a tiered verifier ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredVerifierLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredVerifierLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredVerifierLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered verifier ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierLedger {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub weight: f64,
    pub namespace: String,
    pub discount: f64,
    pub fingerprint: String,
    pub archived: bool,
    pub capacity: i64,
    pub pinned: bool,
    pub status: TieredVerifierLedgerStatus,
}

impl TieredVerifierLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredVerifierLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredVerifierLedgerStatus) -> bool {
        match self.status {
            TieredVerifierLedgerStatus::Proposed => {
                return next == TieredVerifierLedgerStatus::Rejected || next == TieredVerifierLedgerStatus::Withdrawn;
            }
            TieredVerifierLedgerStatus::Approved => {
                return next == TieredVerifierLedgerStatus::Rejected || next == TieredVerifierLedgerStatus::Withdrawn;
            }
            TieredVerifierLedgerStatus::Rejected => {
                return next == TieredVerifierLedgerStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredVerifierLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered verifier ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        (self.weight * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredVerifierLedger {
        TieredVerifierLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            weight: self.weight,
            namespace: self.namespace.clone(),
            discount: self.discount,
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            capacity: self.capacity,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a tiered verifier ledger in status proposed.
pub fn new_tiered_verifier_ledger(code: &str, name: &str) -> TieredVerifierLedger {
    TieredVerifierLedger {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        weight: 0.0,
        namespace: "".to_string(),
        discount: 0.0,
        fingerprint: "".to_string(),
        archived: false,
        capacity: 0,
        pinned: false,
        status: TieredVerifierLedgerStatus::Proposed,
    }
}
