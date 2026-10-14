use super::*;

use acme_kit::text;

/// Where a sink forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SinkForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SinkForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkForecast {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub channel: String,
    pub fingerprint: String,
    pub status: SinkForecastStatus,
}

impl SinkForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkForecastStatus) -> bool {
        match self.status {
            SinkForecastStatus::Requested => {
                return next == SinkForecastStatus::Revoked;
            }
            SinkForecastStatus::Granted => {
                match next {
                    SinkForecastStatus::Revoked | SinkForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkForecastStatus::Revoked => {
                return next == SinkForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "sink forecast", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> SinkForecast {
        SinkForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            channel: self.channel.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a sink forecast in status requested.
pub fn new_sink_forecast(code: &str, label: &str) -> SinkForecast {
    SinkForecast {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        channel: "".to_string(),
        fingerprint: "".to_string(),
        status: SinkForecastStatus::Requested,
    }
}
