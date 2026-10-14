use super::*;

use acme_kit::text;

/// Where a batched policy forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedPolicyForecastStatus {
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

impl std::fmt::Display for BatchedPolicyForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched policy forecast as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyForecast {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub reviewer: String,
    pub ttl_seconds: i64,
    pub status: BatchedPolicyForecastStatus,
}

impl BatchedPolicyForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyForecastStatus) -> bool {
        match self.status {
            BatchedPolicyForecastStatus::Idle => {
                return next == BatchedPolicyForecastStatus::Ready || next == BatchedPolicyForecastStatus::Stopped;
            }
            BatchedPolicyForecastStatus::Warming => {
                return next == BatchedPolicyForecastStatus::Ready || next == BatchedPolicyForecastStatus::Draining;
            }
            BatchedPolicyForecastStatus::Ready => {
                return next == BatchedPolicyForecastStatus::Draining || next == BatchedPolicyForecastStatus::Stopped;
            }
            BatchedPolicyForecastStatus::Draining => {
                return next == BatchedPolicyForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPolicyForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPolicyForecast {
        BatchedPolicyForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            reviewer: self.reviewer.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched policy forecast in status idle.
pub fn new_batched_policy_forecast(code: &str, title: &str) -> BatchedPolicyForecast {
    BatchedPolicyForecast {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        reviewer: "".to_string(),
        ttl_seconds: 0,
        status: BatchedPolicyForecastStatus::Idle,
    }
}
