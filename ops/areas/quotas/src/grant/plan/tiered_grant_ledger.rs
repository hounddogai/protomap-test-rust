use super::*;

use acme_kit::text;

/// Where a tiered grant ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredGrantLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredGrantLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredGrantLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered grant ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantLedger {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub summary: String,
    pub priority: i64,
    pub floor_value: f64,
    pub segment: String,
    pub status: TieredGrantLedgerStatus,
}

impl TieredGrantLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredGrantLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredGrantLedgerStatus) -> bool {
        match self.status {
            TieredGrantLedgerStatus::Requested => {
                match next {
                    TieredGrantLedgerStatus::Granted | TieredGrantLedgerStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredGrantLedgerStatus::Granted => {
                return next == TieredGrantLedgerStatus::Revoked || next == TieredGrantLedgerStatus::Expired;
            }
            TieredGrantLedgerStatus::Revoked => {
                return next == TieredGrantLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredGrantLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered grant ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered grant ledger", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.floor_value * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredGrantLedger {
        TieredGrantLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            summary: self.summary.clone(),
            priority: self.priority,
            floor_value: self.floor_value,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered grant ledger in status requested.
pub fn new_tiered_grant_ledger(code: &str, label: &str) -> TieredGrantLedger {
    TieredGrantLedger {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        summary: "".to_string(),
        priority: 0,
        floor_value: 0.0,
        segment: "".to_string(),
        status: TieredGrantLedgerStatus::Requested,
    }
}
