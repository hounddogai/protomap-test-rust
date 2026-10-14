use super::*;

use acme_kit::text;

/// Where a fallback policy forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPolicyForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackPolicyForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for FallbackPolicyForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback policy forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyForecast {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub score: f64,
    pub duration_minutes: i64,
    pub region: String,
    pub tenant_id: String,
    pub status: FallbackPolicyForecastStatus,
}

impl FallbackPolicyForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPolicyForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPolicyForecastStatus) -> bool {
        match self.status {
            FallbackPolicyForecastStatus::Idle => {
                match next {
                    FallbackPolicyForecastStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPolicyForecastStatus::Warming => {
                return next == FallbackPolicyForecastStatus::Ready;
            }
            FallbackPolicyForecastStatus::Ready => {
                return next == FallbackPolicyForecastStatus::Draining;
            }
            FallbackPolicyForecastStatus::Draining => {
                return next == FallbackPolicyForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPolicyForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback policy forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.score + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPolicyForecast {
        FallbackPolicyForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            score: self.score,
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a fallback policy forecast in status idle.
pub fn new_fallback_policy_forecast(code: &str, name: &str) -> FallbackPolicyForecast {
    FallbackPolicyForecast {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        score: 0.0,
        duration_minutes: 0,
        region: "".to_string(),
        tenant_id: "".to_string(),
        status: FallbackPolicyForecastStatus::Idle,
    }
}
