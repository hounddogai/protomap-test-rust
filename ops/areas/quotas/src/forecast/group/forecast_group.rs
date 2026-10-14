use super::*;

use acme_kit::text;

/// Where a forecast group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ForecastGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ForecastGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast group as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastGroup {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub active: bool,
    pub pinned: bool,
    pub amount: f64,
    pub expires_at: i64,
    pub reviewer: String,
    pub status: ForecastGroupStatus,
}

impl ForecastGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastGroupStatus) -> bool {
        match self.status {
            ForecastGroupStatus::Created => {
                return next == ForecastGroupStatus::Published || next == ForecastGroupStatus::Retired;
            }
            ForecastGroupStatus::Validated => {
                match next {
                    ForecastGroupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastGroupStatus::Published => {
                match next {
                    ForecastGroupStatus::Retired => {
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

    pub fn move_to(&mut self, next: ForecastGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastGroup {
        ForecastGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            active: self.active,
            pinned: self.pinned,
            amount: self.amount,
            expires_at: self.expires_at,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a forecast group in status created.
pub fn new_forecast_group(code: &str, title: &str) -> ForecastGroup {
    ForecastGroup {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        external_ref: "".to_string(),
        duration_minutes: 0,
        active: false,
        pinned: false,
        amount: 0.0,
        expires_at: 0,
        reviewer: "".to_string(),
        status: ForecastGroupStatus::Created,
    }
}
