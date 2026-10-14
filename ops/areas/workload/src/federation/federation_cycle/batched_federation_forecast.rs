use super::*;

use acme_kit::text;

/// Where a batched federation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFederationForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedFederationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedFederationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched federation forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationForecast {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub region: String,
    pub source: String,
    pub currency: String,
    pub threshold: f64,
    pub status: BatchedFederationForecastStatus,
}

impl BatchedFederationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFederationForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFederationForecastStatus) -> bool {
        match self.status {
            BatchedFederationForecastStatus::Proposed => {
                return next == BatchedFederationForecastStatus::Approved;
            }
            BatchedFederationForecastStatus::Approved => {
                match next {
                    BatchedFederationForecastStatus::Rejected | BatchedFederationForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFederationForecastStatus::Rejected => {
                return next == BatchedFederationForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedFederationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched federation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched federation forecast", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> BatchedFederationForecast {
        BatchedFederationForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            region: self.region.clone(),
            source: self.source.clone(),
            currency: self.currency.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched federation forecast in status proposed.
pub fn new_batched_federation_forecast(code: &str, name: &str) -> BatchedFederationForecast {
    BatchedFederationForecast {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        region: "".to_string(),
        source: "".to_string(),
        currency: "".to_string(),
        threshold: 0.0,
        status: BatchedFederationForecastStatus::Proposed,
    }
}
