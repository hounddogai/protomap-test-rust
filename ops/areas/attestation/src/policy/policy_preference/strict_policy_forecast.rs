use super::*;

use acme_kit::text;

/// Where a strict policy forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPolicyForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictPolicyForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictPolicyForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict policy forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyForecast {
    pub code: String,
    pub label: String,
    pub host: String,
    pub description: String,
    pub quantity: i64,
    pub active: bool,
    pub cost: f64,
    pub status: StrictPolicyForecastStatus,
}

impl StrictPolicyForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPolicyForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPolicyForecastStatus) -> bool {
        match self.status {
            StrictPolicyForecastStatus::Created => {
                match next {
                    StrictPolicyForecastStatus::Validated | StrictPolicyForecastStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPolicyForecastStatus::Validated => {
                return next == StrictPolicyForecastStatus::Published || next == StrictPolicyForecastStatus::Retired;
            }
            StrictPolicyForecastStatus::Published => {
                return next == StrictPolicyForecastStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPolicyForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict policy forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict policy forecast", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quantity as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPolicyForecast {
        StrictPolicyForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            description: self.description.clone(),
            quantity: self.quantity,
            active: self.active,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict policy forecast in status created.
pub fn new_strict_policy_forecast(code: &str, label: &str) -> StrictPolicyForecast {
    StrictPolicyForecast {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        description: "".to_string(),
        quantity: 0,
        active: false,
        cost: 0.0,
        status: StrictPolicyForecastStatus::Created,
    }
}
