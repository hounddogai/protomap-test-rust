use super::*;

use acme_kit::text;

/// Where a batched forecast milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedForecastMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedForecastMilestoneStatus {
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

impl std::fmt::Display for BatchedForecastMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched forecast milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastMilestone {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub window_days: i64,
    pub timezone: String,
    pub status: BatchedForecastMilestoneStatus,
}

impl BatchedForecastMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedForecastMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedForecastMilestoneStatus) -> bool {
        match self.status {
            BatchedForecastMilestoneStatus::New => {
                match next {
                    BatchedForecastMilestoneStatus::Queued | BatchedForecastMilestoneStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedForecastMilestoneStatus::Queued => {
                return next == BatchedForecastMilestoneStatus::Failed;
            }
            BatchedForecastMilestoneStatus::Running => {
                return next == BatchedForecastMilestoneStatus::Done;
            }
            BatchedForecastMilestoneStatus::Done => {
                match next {
                    BatchedForecastMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedForecastMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched forecast milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> BatchedForecastMilestone {
        BatchedForecastMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            window_days: self.window_days,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched forecast milestone in status new.
pub fn new_batched_forecast_milestone(code: &str, title: &str) -> BatchedForecastMilestone {
    BatchedForecastMilestone {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        window_days: 0,
        timezone: "".to_string(),
        status: BatchedForecastMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_forecast_milestone("code-870", "title-689");
        assert_eq!(item.status, BatchedForecastMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_forecast_milestone("code-870", "title-689");
        if item.move_to(BatchedForecastMilestoneStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedForecastMilestoneStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_forecast_milestone("code-870", "title-689");
        item.status = BatchedForecastMilestoneStatus::Failed;
        assert!(
            item.move_to(BatchedForecastMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
