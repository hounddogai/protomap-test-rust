use super::*;

use acme_kit::text;

/// Where a default budget burn metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultBudgetBurnMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultBudgetBurnMetricStatus {
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

impl std::fmt::Display for DefaultBudgetBurnMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default budget burn metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnMetric {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub floor_value: f64,
    pub contact_email: String,
    pub currency: String,
    pub status: DefaultBudgetBurnMetricStatus,
}

impl DefaultBudgetBurnMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultBudgetBurnMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultBudgetBurnMetricStatus) -> bool {
        match self.status {
            DefaultBudgetBurnMetricStatus::New => {
                match next {
                    DefaultBudgetBurnMetricStatus::Queued | DefaultBudgetBurnMetricStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultBudgetBurnMetricStatus::Queued => {
                match next {
                    DefaultBudgetBurnMetricStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultBudgetBurnMetricStatus::Running => {
                return next == DefaultBudgetBurnMetricStatus::Done || next == DefaultBudgetBurnMetricStatus::Failed;
            }
            DefaultBudgetBurnMetricStatus::Done => {
                match next {
                    DefaultBudgetBurnMetricStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultBudgetBurnMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default budget burn metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default budget burn metric", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.floor_value - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultBudgetBurnMetric {
        DefaultBudgetBurnMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            floor_value: self.floor_value,
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default budget burn metric in status new.
pub fn new_default_budget_burn_metric(code: &str, label: &str) -> DefaultBudgetBurnMetric {
    DefaultBudgetBurnMetric {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        floor_value: 0.0,
        contact_email: "".to_string(),
        currency: "".to_string(),
        status: DefaultBudgetBurnMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_budget_burn_metric("code-117", "label-907");
        assert_eq!(item.status, DefaultBudgetBurnMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_default_budget_burn_metric("code-117", "label-907");
        if item.move_to(DefaultBudgetBurnMetricStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultBudgetBurnMetricStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_budget_burn_metric("code-117", "label-907");
        item.status = DefaultBudgetBurnMetricStatus::Failed;
        assert!(
            item.move_to(DefaultBudgetBurnMetricStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
