use super::*;

use acme_kit::text;

/// Where a seasonal forecast cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalForecastCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalForecastCycleStatus {
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

impl std::fmt::Display for SeasonalForecastCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal forecast cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastCycle {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub rate: f64,
    pub namespace: String,
    pub discount: f64,
    pub batch_size: i64,
    pub margin: f64,
    pub status: SeasonalForecastCycleStatus,
}

impl SeasonalForecastCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalForecastCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalForecastCycleStatus) -> bool {
        match self.status {
            SeasonalForecastCycleStatus::New => {
                match next {
                    SeasonalForecastCycleStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalForecastCycleStatus::Queued => {
                match next {
                    SeasonalForecastCycleStatus::Running | SeasonalForecastCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalForecastCycleStatus::Running => {
                match next {
                    SeasonalForecastCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalForecastCycleStatus::Done => {
                return next == SeasonalForecastCycleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalForecastCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal forecast cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.batch_size as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalForecastCycle {
        SeasonalForecastCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            rate: self.rate,
            namespace: self.namespace.clone(),
            discount: self.discount,
            batch_size: self.batch_size,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a seasonal forecast cycle in status new.
pub fn new_seasonal_forecast_cycle(code: &str, label: &str) -> SeasonalForecastCycle {
    SeasonalForecastCycle {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        rate: 0.0,
        namespace: "".to_string(),
        discount: 0.0,
        batch_size: 0,
        margin: 0.0,
        status: SeasonalForecastCycleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_forecast_cycle("code-825", "label-597");
        assert_eq!(item.status, SeasonalForecastCycleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_seasonal_forecast_cycle("code-825", "label-597");
        if item.move_to(SeasonalForecastCycleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalForecastCycleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_forecast_cycle("code-825", "label-597");
        item.status = SeasonalForecastCycleStatus::Failed;
        assert!(
            item.move_to(SeasonalForecastCycleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
