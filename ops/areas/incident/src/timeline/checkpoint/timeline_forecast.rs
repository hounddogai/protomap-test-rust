use super::*;

use acme_kit::text;

/// Where a timeline forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TimelineForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TimelineForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineForecast {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub priority: i64,
    pub updated_at: i64,
    pub namespace: String,
    pub timezone: String,
    pub status: TimelineForecastStatus,
}

impl TimelineForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineForecastStatus) -> bool {
        match self.status {
            TimelineForecastStatus::Requested => {
                match next {
                    TimelineForecastStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineForecastStatus::Granted => {
                return next == TimelineForecastStatus::Revoked;
            }
            TimelineForecastStatus::Revoked => {
                match next {
                    TimelineForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: TimelineForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline forecast {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.updated_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineForecast {
        TimelineForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            priority: self.priority,
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a timeline forecast in status requested.
pub fn new_timeline_forecast(code: &str, title: &str) -> TimelineForecast {
    TimelineForecast {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        priority: 0,
        updated_at: 0,
        namespace: "".to_string(),
        timezone: "".to_string(),
        status: TimelineForecastStatus::Requested,
    }
}
