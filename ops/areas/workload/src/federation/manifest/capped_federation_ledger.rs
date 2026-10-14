use super::*;

use acme_kit::text;

/// Where a capped federation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedFederationLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedFederationLedgerStatus {
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

impl std::fmt::Display for CappedFederationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped federation ledger as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationLedger {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub principal: String,
    pub minimum: f64,
    pub cipher: String,
    pub archived: bool,
    pub external_ref: String,
    pub status: CappedFederationLedgerStatus,
}

impl CappedFederationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedFederationLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedFederationLedgerStatus) -> bool {
        match self.status {
            CappedFederationLedgerStatus::Planned => {
                return next == CappedFederationLedgerStatus::Completed
                    || next == CappedFederationLedgerStatus::Cancelled;
            }
            CappedFederationLedgerStatus::Scheduled => {
                match next {
                    CappedFederationLedgerStatus::InProgress | CappedFederationLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFederationLedgerStatus::InProgress => {
                return next == CappedFederationLedgerStatus::Completed
                    || next == CappedFederationLedgerStatus::Cancelled;
            }
            CappedFederationLedgerStatus::Completed => {
                return next == CappedFederationLedgerStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedFederationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped federation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> CappedFederationLedger {
        CappedFederationLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            principal: self.principal.clone(),
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            archived: self.archived,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped federation ledger in status planned.
pub fn new_capped_federation_ledger(code: &str, title: &str) -> CappedFederationLedger {
    CappedFederationLedger {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        principal: "".to_string(),
        minimum: 0.0,
        cipher: "".to_string(),
        archived: false,
        external_ref: "".to_string(),
        status: CappedFederationLedgerStatus::Planned,
    }
}
