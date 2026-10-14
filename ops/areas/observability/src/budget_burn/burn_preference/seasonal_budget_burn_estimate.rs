use super::*;

use acme_kit::text;

/// Where a seasonal budget burn estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalBudgetBurnEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalBudgetBurnEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalBudgetBurnEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal budget burn estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnEstimate {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub key_id: String,
    pub phone: String,
    pub threshold: f64,
    pub score: f64,
    pub ratio: f64,
    pub burst: i64,
    pub status: SeasonalBudgetBurnEstimateStatus,
}

impl SeasonalBudgetBurnEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalBudgetBurnEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalBudgetBurnEstimateStatus) -> bool {
        match self.status {
            SeasonalBudgetBurnEstimateStatus::Planned => {
                match next {
                    SeasonalBudgetBurnEstimateStatus::Scheduled | SeasonalBudgetBurnEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalBudgetBurnEstimateStatus::Scheduled => {
                match next {
                    SeasonalBudgetBurnEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalBudgetBurnEstimateStatus::InProgress => {
                return next == SeasonalBudgetBurnEstimateStatus::Completed;
            }
            SeasonalBudgetBurnEstimateStatus::Completed => {
                match next {
                    SeasonalBudgetBurnEstimateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SeasonalBudgetBurnEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal budget burn estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.score * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalBudgetBurnEstimate {
        SeasonalBudgetBurnEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            key_id: self.key_id.clone(),
            phone: self.phone.clone(),
            threshold: self.threshold,
            score: self.score,
            ratio: self.ratio,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal budget burn estimate in status planned.
pub fn new_seasonal_budget_burn_estimate(code: &str, title: &str) -> SeasonalBudgetBurnEstimate {
    SeasonalBudgetBurnEstimate {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        key_id: "".to_string(),
        phone: "".to_string(),
        threshold: 0.0,
        score: 0.0,
        ratio: 0.0,
        burst: 0,
        status: SeasonalBudgetBurnEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_seasonal_budget_burn_estimate("code-268", "title-417");
        assert_eq!(item.status, SeasonalBudgetBurnEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_seasonal_budget_burn_estimate("code-268", "title-417");
        if item.move_to(SeasonalBudgetBurnEstimateStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalBudgetBurnEstimateStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_seasonal_budget_burn_estimate("code-268", "title-417");
        item.status = SeasonalBudgetBurnEstimateStatus::Cancelled;
        assert!(
            item.move_to(SeasonalBudgetBurnEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
