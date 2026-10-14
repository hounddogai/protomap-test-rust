use super::*;

use acme_kit::text;

/// Where a scheduled limit forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledLimitForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledLimitForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ScheduledLimitForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled limit forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitForecast {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub duration_minutes: i64,
    pub region: String,
    pub key_id: String,
    pub pinned: bool,
    pub currency: String,
    pub cluster: String,
    pub status: ScheduledLimitForecastStatus,
}

impl ScheduledLimitForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledLimitForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledLimitForecastStatus) -> bool {
        match self.status {
            ScheduledLimitForecastStatus::Open => {
                match next {
                    ScheduledLimitForecastStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledLimitForecastStatus::InReview => {
                match next {
                    ScheduledLimitForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledLimitForecastStatus::Resolved => {
                match next {
                    ScheduledLimitForecastStatus::Reopened | ScheduledLimitForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledLimitForecastStatus::Reopened => {
                return next == ScheduledLimitForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledLimitForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled limit forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled limit forecast", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledLimitForecast {
        ScheduledLimitForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            key_id: self.key_id.clone(),
            pinned: self.pinned,
            currency: self.currency.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a scheduled limit forecast in status open.
pub fn new_scheduled_limit_forecast(code: &str, name: &str) -> ScheduledLimitForecast {
    ScheduledLimitForecast {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        duration_minutes: 0,
        region: "".to_string(),
        key_id: "".to_string(),
        pinned: false,
        currency: "".to_string(),
        cluster: "".to_string(),
        status: ScheduledLimitForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_limit_forecast("code-448", "name-477");
        assert_eq!(item.status, ScheduledLimitForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_scheduled_limit_forecast("code-448", "name-477");
        if item.move_to(ScheduledLimitForecastStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledLimitForecastStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_limit_forecast("code-448", "name-477");
        item.status = ScheduledLimitForecastStatus::Archived;
        assert!(
            item.move_to(ScheduledLimitForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
