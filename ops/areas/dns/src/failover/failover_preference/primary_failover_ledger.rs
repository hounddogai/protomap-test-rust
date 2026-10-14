use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary failover ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFailoverLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryFailoverLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryFailoverLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary failover ledger as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverLedger {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub unit: String,
    pub attributes: BTreeMap<String, String>,
    pub quantity: i64,
    pub category: String,
    pub approved: bool,
    pub rate: f64,
    pub amount: f64,
    pub status: PrimaryFailoverLedgerStatus,
}

impl PrimaryFailoverLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFailoverLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFailoverLedgerStatus) -> bool {
        match self.status {
            PrimaryFailoverLedgerStatus::Proposed => {
                return next == PrimaryFailoverLedgerStatus::Rejected;
            }
            PrimaryFailoverLedgerStatus::Approved => {
                return next == PrimaryFailoverLedgerStatus::Rejected || next == PrimaryFailoverLedgerStatus::Withdrawn;
            }
            PrimaryFailoverLedgerStatus::Rejected => {
                match next {
                    PrimaryFailoverLedgerStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryFailoverLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary failover ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quantity as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryFailoverLedger {
        PrimaryFailoverLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            unit: self.unit.clone(),
            attributes: self.attributes.clone(),
            quantity: self.quantity,
            category: self.category.clone(),
            approved: self.approved,
            rate: self.rate,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary failover ledger in status proposed.
pub fn new_primary_failover_ledger(code: &str, title: &str) -> PrimaryFailoverLedger {
    PrimaryFailoverLedger {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        unit: "".to_string(),
        attributes: BTreeMap::new(),
        quantity: 0,
        category: "".to_string(),
        approved: false,
        rate: 0.0,
        amount: 0.0,
        status: PrimaryFailoverLedgerStatus::Proposed,
    }
}
