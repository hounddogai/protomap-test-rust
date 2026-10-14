use super::*;

use acme_kit::text;

/// Where a regional plan metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPlanMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalPlanMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalPlanMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional plan metric as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanMetric {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub threshold: f64,
    pub title: String,
    pub ceiling: f64,
    pub country: String,
    pub maximum: f64,
    pub status: RegionalPlanMetricStatus,
}

impl RegionalPlanMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPlanMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPlanMetricStatus) -> bool {
        match self.status {
            RegionalPlanMetricStatus::Proposed => {
                return next == RegionalPlanMetricStatus::Approved || next == RegionalPlanMetricStatus::Withdrawn;
            }
            RegionalPlanMetricStatus::Approved => {
                return next == RegionalPlanMetricStatus::Rejected || next == RegionalPlanMetricStatus::Withdrawn;
            }
            RegionalPlanMetricStatus::Rejected => {
                return next == RegionalPlanMetricStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalPlanMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional plan metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional plan metric", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.maximum * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPlanMetric {
        RegionalPlanMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            threshold: self.threshold,
            title: self.title.clone(),
            ceiling: self.ceiling,
            country: self.country.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a regional plan metric in status proposed.
pub fn new_regional_plan_metric(code: &str, name: &str) -> RegionalPlanMetric {
    RegionalPlanMetric {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        threshold: 0.0,
        title: "".to_string(),
        ceiling: 0.0,
        country: "".to_string(),
        maximum: 0.0,
        status: RegionalPlanMetricStatus::Proposed,
    }
}
