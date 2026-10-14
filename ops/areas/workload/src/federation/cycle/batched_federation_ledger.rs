use super::*;

use acme_kit::text;

/// Where a batched federation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFederationLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedFederationLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedFederationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched federation ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationLedger {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub contact_email: String,
    pub channel: String,
    pub country: String,
    pub maximum: f64,
    pub rate: f64,
    pub locale: String,
    pub active: bool,
    pub revision: i64,
    pub status: BatchedFederationLedgerStatus,
}

impl BatchedFederationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFederationLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFederationLedgerStatus) -> bool {
        match self.status {
            BatchedFederationLedgerStatus::Requested => {
                return next == BatchedFederationLedgerStatus::Revoked;
            }
            BatchedFederationLedgerStatus::Granted => {
                match next {
                    BatchedFederationLedgerStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFederationLedgerStatus::Revoked => {
                return next == BatchedFederationLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedFederationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched federation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched federation ledger", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.maximum + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedFederationLedger {
        BatchedFederationLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            contact_email: self.contact_email.clone(),
            channel: self.channel.clone(),
            country: self.country.clone(),
            maximum: self.maximum,
            rate: self.rate,
            locale: self.locale.clone(),
            active: self.active,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched federation ledger in status requested.
pub fn new_batched_federation_ledger(code: &str, name: &str) -> BatchedFederationLedger {
    BatchedFederationLedger {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        contact_email: "".to_string(),
        channel: "".to_string(),
        country: "".to_string(),
        maximum: 0.0,
        rate: 0.0,
        locale: "".to_string(),
        active: false,
        revision: 0,
        status: BatchedFederationLedgerStatus::Requested,
    }
}
