use super::*;

use acme_kit::text;

/// Where a rollout forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RolloutForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RolloutForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout forecast as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutForecast {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub ratio: f64,
    pub budget: f64,
    pub updated_at: i64,
    pub audience: String,
    pub priority: i64,
    pub retries: i64,
    pub port: i64,
    pub version: i64,
    pub status: RolloutForecastStatus,
}

impl RolloutForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutForecastStatus) -> bool {
        match self.status {
            RolloutForecastStatus::Proposed => {
                return next == RolloutForecastStatus::Withdrawn;
            }
            RolloutForecastStatus::Approved => {
                match next {
                    RolloutForecastStatus::Rejected | RolloutForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutForecastStatus::Rejected => {
                return next == RolloutForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.budget * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutForecast {
        RolloutForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            ratio: self.ratio,
            budget: self.budget,
            updated_at: self.updated_at,
            audience: self.audience.clone(),
            priority: self.priority,
            retries: self.retries,
            port: self.port,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rollout forecast in status proposed.
pub fn new_rollout_forecast(code: &str, title: &str) -> RolloutForecast {
    RolloutForecast {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        ratio: 0.0,
        budget: 0.0,
        updated_at: 0,
        audience: "".to_string(),
        priority: 0,
        retries: 0,
        port: 0,
        version: 0,
        status: RolloutForecastStatus::Proposed,
    }
}
