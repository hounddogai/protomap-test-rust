use super::*;

use acme_kit::text;

/// Where a dashboard estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DashboardEstimateStatus {
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

impl std::fmt::Display for DashboardEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardEstimate {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub category: String,
    pub archived: bool,
    pub cipher: String,
    pub status: DashboardEstimateStatus,
}

impl DashboardEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardEstimateStatus) -> bool {
        match self.status {
            DashboardEstimateStatus::Planned => {
                match next {
                    DashboardEstimateStatus::Completed | DashboardEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardEstimateStatus::Scheduled => {
                match next {
                    DashboardEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardEstimateStatus::InProgress => {
                match next {
                    DashboardEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardEstimateStatus::Completed => {
                match next {
                    DashboardEstimateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DashboardEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> DashboardEstimate {
        DashboardEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            category: self.category.clone(),
            archived: self.archived,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a dashboard estimate in status planned.
pub fn new_dashboard_estimate(code: &str, title: &str) -> DashboardEstimate {
    DashboardEstimate {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        category: "".to_string(),
        archived: false,
        cipher: "".to_string(),
        status: DashboardEstimateStatus::Planned,
    }
}
