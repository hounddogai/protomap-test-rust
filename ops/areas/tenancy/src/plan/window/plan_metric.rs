use super::*;

use acme_kit::text;

/// Where a plan metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PlanMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PlanMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMetric {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub limit: i64,
    pub key_id: String,
    pub owner_id: String,
    pub port: i64,
    pub tenant_id: String,
    pub status: PlanMetricStatus,
}

impl PlanMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanMetricStatus) -> bool {
        match self.status {
            PlanMetricStatus::Open => {
                match next {
                    PlanMetricStatus::Resolved | PlanMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanMetricStatus::InReview => {
                return next == PlanMetricStatus::Resolved;
            }
            PlanMetricStatus::Resolved => {
                return next == PlanMetricStatus::Reopened || next == PlanMetricStatus::Archived;
            }
            PlanMetricStatus::Reopened => {
                match next {
                    PlanMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: PlanMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan metric", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ratio + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanMetric {
        PlanMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            limit: self.limit,
            key_id: self.key_id.clone(),
            owner_id: self.owner_id.clone(),
            port: self.port,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a plan metric in status open.
pub fn new_plan_metric(code: &str, title: &str) -> PlanMetric {
    PlanMetric {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        limit: 0,
        key_id: "".to_string(),
        owner_id: "".to_string(),
        port: 0,
        tenant_id: "".to_string(),
        status: PlanMetricStatus::Open,
    }
}
