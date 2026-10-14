use super::*;

use acme_kit::text;

/// Where a seasonal dashboard estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDashboardEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalDashboardEstimateStatus {
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

impl std::fmt::Display for SeasonalDashboardEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal dashboard estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardEstimate {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub summary: String,
    pub port: i64,
    pub region: String,
    pub description: String,
    pub budget: f64,
    pub external_ref: String,
    pub status: SeasonalDashboardEstimateStatus,
}

impl SeasonalDashboardEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDashboardEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDashboardEstimateStatus) -> bool {
        match self.status {
            SeasonalDashboardEstimateStatus::Open => {
                match next {
                    SeasonalDashboardEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDashboardEstimateStatus::InReview => {
                match next {
                    SeasonalDashboardEstimateStatus::Reopened | SeasonalDashboardEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDashboardEstimateStatus::Resolved => {
                match next {
                    SeasonalDashboardEstimateStatus::Reopened | SeasonalDashboardEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDashboardEstimateStatus::Reopened => {
                return next == SeasonalDashboardEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalDashboardEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal dashboard estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal dashboard estimate", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.port as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDashboardEstimate {
        SeasonalDashboardEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            summary: self.summary.clone(),
            port: self.port,
            region: self.region.clone(),
            description: self.description.clone(),
            budget: self.budget,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a seasonal dashboard estimate in status open.
pub fn new_seasonal_dashboard_estimate(code: &str, label: &str) -> SeasonalDashboardEstimate {
    SeasonalDashboardEstimate {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        summary: "".to_string(),
        port: 0,
        region: "".to_string(),
        description: "".to_string(),
        budget: 0.0,
        external_ref: "".to_string(),
        status: SeasonalDashboardEstimateStatus::Open,
    }
}
