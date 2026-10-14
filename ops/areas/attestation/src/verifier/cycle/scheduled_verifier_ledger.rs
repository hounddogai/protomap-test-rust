use super::*;

use acme_kit::text;

/// Where a scheduled verifier ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledVerifierLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledVerifierLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledVerifierLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled verifier ledger as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierLedger {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub owner_id: String,
    pub external_ref: String,
    pub country: String,
    pub archived: bool,
    pub discount: f64,
    pub duration_minutes: i64,
    pub status: ScheduledVerifierLedgerStatus,
}

impl ScheduledVerifierLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledVerifierLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledVerifierLedgerStatus) -> bool {
        match self.status {
            ScheduledVerifierLedgerStatus::Proposed => {
                return next == ScheduledVerifierLedgerStatus::Approved
                    || next == ScheduledVerifierLedgerStatus::Withdrawn;
            }
            ScheduledVerifierLedgerStatus::Approved => {
                return next == ScheduledVerifierLedgerStatus::Rejected;
            }
            ScheduledVerifierLedgerStatus::Rejected => {
                match next {
                    ScheduledVerifierLedgerStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledVerifierLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled verifier ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled verifier ledger", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.version as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledVerifierLedger {
        ScheduledVerifierLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            owner_id: self.owner_id.clone(),
            external_ref: self.external_ref.clone(),
            country: self.country.clone(),
            archived: self.archived,
            discount: self.discount,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled verifier ledger in status proposed.
pub fn new_scheduled_verifier_ledger(code: &str, title: &str) -> ScheduledVerifierLedger {
    ScheduledVerifierLedger {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        owner_id: "".to_string(),
        external_ref: "".to_string(),
        country: "".to_string(),
        archived: false,
        discount: 0.0,
        duration_minutes: 0,
        status: ScheduledVerifierLedgerStatus::Proposed,
    }
}
