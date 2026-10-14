use super::*;

use acme_kit::text;

/// Where a forecast ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ForecastLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ForecastLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastLedger {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub timezone: String,
    pub title: String,
    pub score: f64,
    pub status: ForecastLedgerStatus,
}

impl ForecastLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastLedgerStatus) -> bool {
        match self.status {
            ForecastLedgerStatus::Requested => {
                return next == ForecastLedgerStatus::Revoked;
            }
            ForecastLedgerStatus::Granted => {
                match next {
                    ForecastLedgerStatus::Revoked | ForecastLedgerStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastLedgerStatus::Revoked => {
                match next {
                    ForecastLedgerStatus::Expired => {
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

    pub fn move_to(&mut self, next: ForecastLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ForecastLedger {
        ForecastLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            timezone: self.timezone.clone(),
            title: self.title.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a forecast ledger in status requested.
pub fn new_forecast_ledger(code: &str, label: &str) -> ForecastLedger {
    ForecastLedger {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        timezone: "".to_string(),
        title: "".to_string(),
        score: 0.0,
        status: ForecastLedgerStatus::Requested,
    }
}
