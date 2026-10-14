use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual page forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPageForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualPageForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualPageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual page forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageForecast {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub duration_minutes: i64,
    pub attributes: BTreeMap<String, String>,
    pub region: String,
    pub timezone: String,
    pub floor_value: f64,
    pub segment: String,
    pub fingerprint: String,
    pub margin: f64,
    pub status: ManualPageForecastStatus,
}

impl ManualPageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPageForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPageForecastStatus) -> bool {
        match self.status {
            ManualPageForecastStatus::Requested => {
                return next == ManualPageForecastStatus::Revoked || next == ManualPageForecastStatus::Expired;
            }
            ManualPageForecastStatus::Granted => {
                return next == ManualPageForecastStatus::Revoked;
            }
            ManualPageForecastStatus::Revoked => {
                return next == ManualPageForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualPageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual page forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_exposure(&self) -> f64 {
        (self.floor_value * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPageForecast {
        ManualPageForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            duration_minutes: self.duration_minutes,
            attributes: self.attributes.clone(),
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            floor_value: self.floor_value,
            segment: self.segment.clone(),
            fingerprint: self.fingerprint.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual page forecast in status requested.
pub fn new_manual_page_forecast(code: &str, title: &str) -> ManualPageForecast {
    ManualPageForecast {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        duration_minutes: 0,
        attributes: BTreeMap::new(),
        region: "".to_string(),
        timezone: "".to_string(),
        floor_value: 0.0,
        segment: "".to_string(),
        fingerprint: "".to_string(),
        margin: 0.0,
        status: ManualPageForecastStatus::Requested,
    }
}
