use super::*;

use acme_kit::text;

/// Where a rolling forecast policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingForecastPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingForecastPolicyStatus {
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

impl std::fmt::Display for RollingForecastPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling forecast policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingForecastPolicy {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub quantity: i64,
    pub country: String,
    pub burst: i64,
    pub version: i64,
    pub locale: String,
    pub status: RollingForecastPolicyStatus,
}

impl RollingForecastPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingForecastPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingForecastPolicyStatus) -> bool {
        match self.status {
            RollingForecastPolicyStatus::New => {
                match next {
                    RollingForecastPolicyStatus::Running | RollingForecastPolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingForecastPolicyStatus::Queued => {
                match next {
                    RollingForecastPolicyStatus::Running | RollingForecastPolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingForecastPolicyStatus::Running => {
                return next == RollingForecastPolicyStatus::Done || next == RollingForecastPolicyStatus::Failed;
            }
            RollingForecastPolicyStatus::Done => {
                match next {
                    RollingForecastPolicyStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingForecastPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling forecast policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling forecast policy", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.burst as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingForecastPolicy {
        RollingForecastPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            country: self.country.clone(),
            burst: self.burst,
            version: self.version,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling forecast policy in status new.
pub fn new_rolling_forecast_policy(code: &str, name: &str) -> RollingForecastPolicy {
    RollingForecastPolicy {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        quantity: 0,
        country: "".to_string(),
        burst: 0,
        version: 0,
        locale: "".to_string(),
        status: RollingForecastPolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_forecast_policy("code-428", "name-746");
        assert_eq!(item.status, RollingForecastPolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rolling_forecast_policy("code-428", "name-746");
        if item.move_to(RollingForecastPolicyStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingForecastPolicyStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_forecast_policy("code-428", "name-746");
        item.status = RollingForecastPolicyStatus::Failed;
        assert!(
            item.move_to(RollingForecastPolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
