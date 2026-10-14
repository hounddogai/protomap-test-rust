use super::*;

use acme_kit::text;

/// Where a credential forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CredentialForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CredentialForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialForecast {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub duration_minutes: i64,
    pub audience: String,
    pub notes: String,
    pub summary: String,
    pub minimum: f64,
    pub ceiling: f64,
    pub status: CredentialForecastStatus,
}

impl CredentialForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialForecastStatus) -> bool {
        match self.status {
            CredentialForecastStatus::New => {
                return next == CredentialForecastStatus::Running;
            }
            CredentialForecastStatus::Queued => {
                match next {
                    CredentialForecastStatus::Running | CredentialForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialForecastStatus::Running => {
                match next {
                    CredentialForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialForecastStatus::Done => {
                return next == CredentialForecastStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential forecast", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.ceiling * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialForecast {
        CredentialForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            duration_minutes: self.duration_minutes,
            audience: self.audience.clone(),
            notes: self.notes.clone(),
            summary: self.summary.clone(),
            minimum: self.minimum,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a credential forecast in status new.
pub fn new_credential_forecast(code: &str, name: &str) -> CredentialForecast {
    CredentialForecast {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        duration_minutes: 0,
        audience: "".to_string(),
        notes: "".to_string(),
        summary: "".to_string(),
        minimum: 0.0,
        ceiling: 0.0,
        status: CredentialForecastStatus::New,
    }
}
