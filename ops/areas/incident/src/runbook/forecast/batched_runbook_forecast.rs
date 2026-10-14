use super::*;

use acme_kit::text;

/// Where a batched runbook forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRunbookForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedRunbookForecastStatus {
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

impl std::fmt::Display for BatchedRunbookForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched runbook forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookForecast {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub name: String,
    pub currency: String,
    pub weight: f64,
    pub rotation_days: i64,
    pub status: BatchedRunbookForecastStatus,
}

impl BatchedRunbookForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRunbookForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRunbookForecastStatus) -> bool {
        match self.status {
            BatchedRunbookForecastStatus::New => {
                match next {
                    BatchedRunbookForecastStatus::Queued | BatchedRunbookForecastStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRunbookForecastStatus::Queued => {
                match next {
                    BatchedRunbookForecastStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRunbookForecastStatus::Running => {
                return next == BatchedRunbookForecastStatus::Done || next == BatchedRunbookForecastStatus::Failed;
            }
            BatchedRunbookForecastStatus::Done => {
                match next {
                    BatchedRunbookForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedRunbookForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched runbook forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.weight * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRunbookForecast {
        BatchedRunbookForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            name: self.name.clone(),
            currency: self.currency.clone(),
            weight: self.weight,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched runbook forecast in status new.
pub fn new_batched_runbook_forecast(code: &str, label: &str) -> BatchedRunbookForecast {
    BatchedRunbookForecast {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        name: "".to_string(),
        currency: "".to_string(),
        weight: 0.0,
        rotation_days: 0,
        status: BatchedRunbookForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_runbook_forecast("code-642", "label-938");
        assert_eq!(item.status, BatchedRunbookForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_runbook_forecast("code-642", "label-938");
        if item.move_to(BatchedRunbookForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRunbookForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_runbook_forecast("code-642", "label-938");
        item.status = BatchedRunbookForecastStatus::Failed;
        assert!(
            item.move_to(BatchedRunbookForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
