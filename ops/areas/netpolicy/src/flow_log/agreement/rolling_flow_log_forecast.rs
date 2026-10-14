use super::*;

use acme_kit::text;

/// Where a rolling flow log forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFlowLogForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingFlowLogForecastStatus {
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

impl std::fmt::Display for RollingFlowLogForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling flow log forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogForecast {
    pub code: String,
    pub name: String,
    pub host: String,
    pub cluster: String,
    pub updated_at: i64,
    pub principal: String,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub starts_at: i64,
    pub status: RollingFlowLogForecastStatus,
}

impl RollingFlowLogForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFlowLogForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFlowLogForecastStatus) -> bool {
        match self.status {
            RollingFlowLogForecastStatus::Idle => {
                return next == RollingFlowLogForecastStatus::Warming || next == RollingFlowLogForecastStatus::Draining;
            }
            RollingFlowLogForecastStatus::Warming => {
                return next == RollingFlowLogForecastStatus::Ready || next == RollingFlowLogForecastStatus::Draining;
            }
            RollingFlowLogForecastStatus::Ready => {
                return next == RollingFlowLogForecastStatus::Stopped;
            }
            RollingFlowLogForecastStatus::Draining => {
                return next == RollingFlowLogForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingFlowLogForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling flow log forecast {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingFlowLogForecast {
        RollingFlowLogForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            cluster: self.cluster.clone(),
            updated_at: self.updated_at,
            principal: self.principal.clone(),
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling flow log forecast in status idle.
pub fn new_rolling_flow_log_forecast(code: &str, name: &str) -> RollingFlowLogForecast {
    RollingFlowLogForecast {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        cluster: "".to_string(),
        updated_at: 0,
        principal: "".to_string(),
        priority: 0,
        ttl_seconds: 0,
        starts_at: 0,
        status: RollingFlowLogForecastStatus::Idle,
    }
}
