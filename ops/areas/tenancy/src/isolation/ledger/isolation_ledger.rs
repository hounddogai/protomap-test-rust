use super::*;

use acme_kit::text;

/// Where a isolation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IsolationLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IsolationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationLedger {
    pub code: String,
    pub name: String,
    pub description: String,
    pub rate: f64,
    pub weight: f64,
    pub title: String,
    pub minimum: f64,
    pub fingerprint: String,
    pub duration_minutes: i64,
    pub status: IsolationLedgerStatus,
}

impl IsolationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationLedgerStatus) -> bool {
        match self.status {
            IsolationLedgerStatus::Requested => {
                match next {
                    IsolationLedgerStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationLedgerStatus::Granted => {
                return next == IsolationLedgerStatus::Revoked || next == IsolationLedgerStatus::Expired;
            }
            IsolationLedgerStatus::Revoked => {
                return next == IsolationLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.minimum * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationLedger {
        IsolationLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            rate: self.rate,
            weight: self.weight,
            title: self.title.clone(),
            minimum: self.minimum,
            fingerprint: self.fingerprint.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a isolation ledger in status requested.
pub fn new_isolation_ledger(code: &str, name: &str) -> IsolationLedger {
    IsolationLedger {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        rate: 0.0,
        weight: 0.0,
        title: "".to_string(),
        minimum: 0.0,
        fingerprint: "".to_string(),
        duration_minutes: 0,
        status: IsolationLedgerStatus::Requested,
    }
}
