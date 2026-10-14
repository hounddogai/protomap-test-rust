use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling budget burn metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingBudgetBurnMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingBudgetBurnMetricStatus {
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

impl std::fmt::Display for RollingBudgetBurnMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling budget burn metric as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetric {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub amount: f64,
    pub visible: bool,
    pub phone: String,
    pub status: RollingBudgetBurnMetricStatus,
}

impl RollingBudgetBurnMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingBudgetBurnMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingBudgetBurnMetricStatus) -> bool {
        match self.status {
            RollingBudgetBurnMetricStatus::New => {
                return next == RollingBudgetBurnMetricStatus::Queued || next == RollingBudgetBurnMetricStatus::Running;
            }
            RollingBudgetBurnMetricStatus::Queued => {
                match next {
                    RollingBudgetBurnMetricStatus::Done | RollingBudgetBurnMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBudgetBurnMetricStatus::Running => {
                match next {
                    RollingBudgetBurnMetricStatus::Done | RollingBudgetBurnMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBudgetBurnMetricStatus::Done => {
                match next {
                    RollingBudgetBurnMetricStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingBudgetBurnMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling budget burn metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_load(&self) -> f64 {
        ((self.window_days as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingBudgetBurnMetric {
        RollingBudgetBurnMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            amount: self.amount,
            visible: self.visible,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling budget burn metric in status new.
pub fn new_rolling_budget_burn_metric(code: &str, title: &str) -> RollingBudgetBurnMetric {
    RollingBudgetBurnMetric {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        attributes: BTreeMap::new(),
        window_days: 0,
        amount: 0.0,
        visible: false,
        phone: "".to_string(),
        status: RollingBudgetBurnMetricStatus::New,
    }
}
