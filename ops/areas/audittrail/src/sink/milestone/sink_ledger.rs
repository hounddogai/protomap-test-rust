use super::*;

use acme_kit::text;

/// Where a sink ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SinkLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SinkLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkLedger {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub grace_days: i64,
    pub active: bool,
    pub status: SinkLedgerStatus,
}

impl SinkLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkLedgerStatus) -> bool {
        match self.status {
            SinkLedgerStatus::Proposed => {
                return next == SinkLedgerStatus::Withdrawn;
            }
            SinkLedgerStatus::Approved => {
                return next == SinkLedgerStatus::Rejected || next == SinkLedgerStatus::Withdrawn;
            }
            SinkLedgerStatus::Rejected => {
                return next == SinkLedgerStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkLedger {
        SinkLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            grace_days: self.grace_days,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a sink ledger in status proposed.
pub fn new_sink_ledger(code: &str, name: &str) -> SinkLedger {
    SinkLedger {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        grace_days: 0,
        active: false,
        status: SinkLedgerStatus::Proposed,
    }
}
