use super::*;

use acme_kit::text;

/// Where a scheduled dashboard record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDashboardRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledDashboardRecordStatus {
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

impl std::fmt::Display for ScheduledDashboardRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled dashboard record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRecord {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub cluster: String,
    pub phone: String,
    pub quantity: i64,
    pub limit: i64,
    pub country: String,
    pub batch_size: i64,
    pub status: ScheduledDashboardRecordStatus,
}

impl ScheduledDashboardRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDashboardRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDashboardRecordStatus) -> bool {
        match self.status {
            ScheduledDashboardRecordStatus::Open => {
                match next {
                    ScheduledDashboardRecordStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDashboardRecordStatus::InReview => {
                match next {
                    ScheduledDashboardRecordStatus::Reopened | ScheduledDashboardRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDashboardRecordStatus::Resolved => {
                return next == ScheduledDashboardRecordStatus::Reopened
                    || next == ScheduledDashboardRecordStatus::Archived;
            }
            ScheduledDashboardRecordStatus::Reopened => {
                return next == ScheduledDashboardRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledDashboardRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled dashboard record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled dashboard record", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quantity as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDashboardRecord {
        ScheduledDashboardRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            cluster: self.cluster.clone(),
            phone: self.phone.clone(),
            quantity: self.quantity,
            limit: self.limit,
            country: self.country.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a scheduled dashboard record in status open.
pub fn new_scheduled_dashboard_record(code: &str, label: &str) -> ScheduledDashboardRecord {
    ScheduledDashboardRecord {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        cluster: "".to_string(),
        phone: "".to_string(),
        quantity: 0,
        limit: 0,
        country: "".to_string(),
        batch_size: 0,
        status: ScheduledDashboardRecordStatus::Open,
    }
}
