use super::*;

use acme_kit::text;

/// Where a failover forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FailoverForecastStatus {
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

impl std::fmt::Display for FailoverForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover forecast as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverForecast {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub unit: String,
    pub description: String,
    pub status: FailoverForecastStatus,
}

impl FailoverForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverForecastStatus) -> bool {
        match self.status {
            FailoverForecastStatus::New => {
                match next {
                    FailoverForecastStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverForecastStatus::Queued => {
                match next {
                    FailoverForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverForecastStatus::Running => {
                return next == FailoverForecastStatus::Done || next == FailoverForecastStatus::Failed;
            }
            FailoverForecastStatus::Done => {
                return next == FailoverForecastStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> FailoverForecast {
        FailoverForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            unit: self.unit.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a failover forecast in status new.
pub fn new_failover_forecast(code: &str, name: &str) -> FailoverForecast {
    FailoverForecast {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        unit: "".to_string(),
        description: "".to_string(),
        status: FailoverForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_failover_forecast("code-104", "name-796");
        assert_eq!(item.status, FailoverForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_failover_forecast("code-104", "name-796");
        if item.move_to(FailoverForecastStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverForecastStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_failover_forecast("code-104", "name-796");
        item.status = FailoverForecastStatus::Failed;
        assert!(
            item.move_to(FailoverForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
