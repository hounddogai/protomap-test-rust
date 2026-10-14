use super::*;

use acme_kit::text;

/// Where a forecast preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ForecastPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast preference as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPreference {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub country: String,
    pub ratio: f64,
    pub updated_at: i64,
    pub status: ForecastPreferenceStatus,
}

impl ForecastPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastPreferenceStatus) -> bool {
        match self.status {
            ForecastPreferenceStatus::Draft => {
                match next {
                    ForecastPreferenceStatus::Active | ForecastPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastPreferenceStatus::Pending => {
                return next == ForecastPreferenceStatus::Active || next == ForecastPreferenceStatus::Suspended;
            }
            ForecastPreferenceStatus::Active => {
                return next == ForecastPreferenceStatus::Suspended || next == ForecastPreferenceStatus::Closed;
            }
            ForecastPreferenceStatus::Suspended => {
                return next == ForecastPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.ratio * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastPreference {
        ForecastPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            country: self.country.clone(),
            ratio: self.ratio,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a forecast preference in status draft.
pub fn new_forecast_preference(code: &str, label: &str) -> ForecastPreference {
    ForecastPreference {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        country: "".to_string(),
        ratio: 0.0,
        updated_at: 0,
        status: ForecastPreferenceStatus::Draft,
    }
}
