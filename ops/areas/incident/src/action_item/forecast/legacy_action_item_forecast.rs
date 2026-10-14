use super::*;

use acme_kit::text;

/// Where a legacy action item forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyActionItemForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyActionItemForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyActionItemForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy action item forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemForecast {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub attempts: i64,
    pub minimum: f64,
    pub grace_days: i64,
    pub principal: String,
    pub timezone: String,
    pub batch_size: i64,
    pub status: LegacyActionItemForecastStatus,
}

impl LegacyActionItemForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyActionItemForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyActionItemForecastStatus) -> bool {
        match self.status {
            LegacyActionItemForecastStatus::Requested => {
                return next == LegacyActionItemForecastStatus::Expired;
            }
            LegacyActionItemForecastStatus::Granted => {
                return next == LegacyActionItemForecastStatus::Revoked
                    || next == LegacyActionItemForecastStatus::Expired;
            }
            LegacyActionItemForecastStatus::Revoked => {
                match next {
                    LegacyActionItemForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyActionItemForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy action item forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy action item forecast", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.batch_size as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyActionItemForecast {
        LegacyActionItemForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            attempts: self.attempts,
            minimum: self.minimum,
            grace_days: self.grace_days,
            principal: self.principal.clone(),
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy action item forecast in status requested.
pub fn new_legacy_action_item_forecast(code: &str, name: &str) -> LegacyActionItemForecast {
    LegacyActionItemForecast {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        attempts: 0,
        minimum: 0.0,
        grace_days: 0,
        principal: "".to_string(),
        timezone: "".to_string(),
        batch_size: 0,
        status: LegacyActionItemForecastStatus::Requested,
    }
}
