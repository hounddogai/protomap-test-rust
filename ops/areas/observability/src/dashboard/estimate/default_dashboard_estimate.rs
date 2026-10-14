use super::*;

use acme_kit::text;

/// Where a default dashboard estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDashboardEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultDashboardEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultDashboardEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default dashboard estimate as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardEstimate {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub checksum: String,
    pub phone: String,
    pub margin: f64,
    pub status: DefaultDashboardEstimateStatus,
}

impl DefaultDashboardEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDashboardEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDashboardEstimateStatus) -> bool {
        match self.status {
            DefaultDashboardEstimateStatus::Proposed => {
                match next {
                    DefaultDashboardEstimateStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDashboardEstimateStatus::Approved => {
                return next == DefaultDashboardEstimateStatus::Rejected
                    || next == DefaultDashboardEstimateStatus::Withdrawn;
            }
            DefaultDashboardEstimateStatus::Rejected => {
                return next == DefaultDashboardEstimateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultDashboardEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default dashboard estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.margin - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDashboardEstimate {
        DefaultDashboardEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default dashboard estimate in status proposed.
pub fn new_default_dashboard_estimate(code: &str, title: &str) -> DefaultDashboardEstimate {
    DefaultDashboardEstimate {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        checksum: "".to_string(),
        phone: "".to_string(),
        margin: 0.0,
        status: DefaultDashboardEstimateStatus::Proposed,
    }
}
