use super::*;

use acme_kit::text;

/// Where a manual allowlist forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAllowlistForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualAllowlistForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualAllowlistForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual allowlist forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistForecast {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub score: f64,
    pub unit: String,
    pub checksum: String,
    pub host: String,
    pub reviewer: String,
    pub grace_days: i64,
    pub limit: i64,
    pub status: ManualAllowlistForecastStatus,
}

impl ManualAllowlistForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAllowlistForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAllowlistForecastStatus) -> bool {
        match self.status {
            ManualAllowlistForecastStatus::Requested => {
                match next {
                    ManualAllowlistForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAllowlistForecastStatus::Granted => {
                return next == ManualAllowlistForecastStatus::Revoked
                    || next == ManualAllowlistForecastStatus::Expired;
            }
            ManualAllowlistForecastStatus::Revoked => {
                return next == ManualAllowlistForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualAllowlistForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual allowlist forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.score - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualAllowlistForecast {
        ManualAllowlistForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            score: self.score,
            unit: self.unit.clone(),
            checksum: self.checksum.clone(),
            host: self.host.clone(),
            reviewer: self.reviewer.clone(),
            grace_days: self.grace_days,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual allowlist forecast in status requested.
pub fn new_manual_allowlist_forecast(code: &str, title: &str) -> ManualAllowlistForecast {
    ManualAllowlistForecast {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        score: 0.0,
        unit: "".to_string(),
        checksum: "".to_string(),
        host: "".to_string(),
        reviewer: "".to_string(),
        grace_days: 0,
        limit: 0,
        status: ManualAllowlistForecastStatus::Requested,
    }
}
