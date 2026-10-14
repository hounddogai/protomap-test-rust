use super::*;

use acme_kit::text;

/// Where a credential ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CredentialLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CredentialLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedger {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub source: String,
    pub external_ref: String,
    pub capacity: i64,
    pub quantity: i64,
    pub status: CredentialLedgerStatus,
}

impl CredentialLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialLedgerStatus) -> bool {
        match self.status {
            CredentialLedgerStatus::New => {
                match next {
                    CredentialLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialLedgerStatus::Queued => {
                return next == CredentialLedgerStatus::Done || next == CredentialLedgerStatus::Failed;
            }
            CredentialLedgerStatus::Running => {
                return next == CredentialLedgerStatus::Done || next == CredentialLedgerStatus::Failed;
            }
            CredentialLedgerStatus::Done => {
                return next == CredentialLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential ledger", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.quantity as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialLedger {
        CredentialLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            capacity: self.capacity,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a credential ledger in status new.
pub fn new_credential_ledger(code: &str, label: &str) -> CredentialLedger {
    CredentialLedger {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        source: "".to_string(),
        external_ref: "".to_string(),
        capacity: 0,
        quantity: 0,
        status: CredentialLedgerStatus::New,
    }
}
