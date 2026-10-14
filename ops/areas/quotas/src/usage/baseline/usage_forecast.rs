use super::*;

use acme_kit::text;

/// Where a usage forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl UsageForecastStatus {
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

impl std::fmt::Display for UsageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageForecast {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub revision: i64,
    pub expires_at: i64,
    pub status: UsageForecastStatus,
}

impl UsageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageForecastStatus) -> bool {
        match self.status {
            UsageForecastStatus::New => {
                return next == UsageForecastStatus::Queued;
            }
            UsageForecastStatus::Queued => {
                match next {
                    UsageForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageForecastStatus::Running => {
                return next == UsageForecastStatus::Done;
            }
            UsageForecastStatus::Done => {
                match next {
                    UsageForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: UsageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "usage forecast", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.expires_at as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageForecast {
        UsageForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            revision: self.revision,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a usage forecast in status new.
pub fn new_usage_forecast(code: &str, name: &str) -> UsageForecast {
    UsageForecast {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        revision: 0,
        expires_at: 0,
        status: UsageForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_usage_forecast("code-700", "name-19");
        assert_eq!(item.status, UsageForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_usage_forecast("code-700", "name-19");
        if item.move_to(UsageForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_usage_forecast("code-700", "name-19");
        item.status = UsageForecastStatus::Failed;
        assert!(item.move_to(UsageForecastStatus::New).is_err(), "expected move to fail");
    }
}
