use super::*;

use acme_kit::text;

/// Where a weighted budget burn metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedBudgetBurnMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedBudgetBurnMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for WeightedBudgetBurnMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted budget burn metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetric {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub revision: i64,
    pub active: bool,
    pub floor_value: f64,
    pub maximum: f64,
    pub status: WeightedBudgetBurnMetricStatus,
}

impl WeightedBudgetBurnMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedBudgetBurnMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedBudgetBurnMetricStatus) -> bool {
        match self.status {
            WeightedBudgetBurnMetricStatus::Idle => {
                return next == WeightedBudgetBurnMetricStatus::Ready
                    || next == WeightedBudgetBurnMetricStatus::Draining;
            }
            WeightedBudgetBurnMetricStatus::Warming => {
                match next {
                    WeightedBudgetBurnMetricStatus::Ready | WeightedBudgetBurnMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnMetricStatus::Ready => {
                return next == WeightedBudgetBurnMetricStatus::Draining
                    || next == WeightedBudgetBurnMetricStatus::Stopped;
            }
            WeightedBudgetBurnMetricStatus::Draining => {
                match next {
                    WeightedBudgetBurnMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: WeightedBudgetBurnMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted budget burn metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.minimum - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedBudgetBurnMetric {
        WeightedBudgetBurnMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            revision: self.revision,
            active: self.active,
            floor_value: self.floor_value,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted budget burn metric in status idle.
pub fn new_weighted_budget_burn_metric(code: &str, title: &str) -> WeightedBudgetBurnMetric {
    WeightedBudgetBurnMetric {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        revision: 0,
        active: false,
        floor_value: 0.0,
        maximum: 0.0,
        status: WeightedBudgetBurnMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_budget_burn_metric("code-628", "title-794");
        assert_eq!(item.status, WeightedBudgetBurnMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_weighted_budget_burn_metric("code-628", "title-794");
        if item.move_to(WeightedBudgetBurnMetricStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedBudgetBurnMetricStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_budget_burn_metric("code-628", "title-794");
        item.status = WeightedBudgetBurnMetricStatus::Stopped;
        assert!(
            item.move_to(WeightedBudgetBurnMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
