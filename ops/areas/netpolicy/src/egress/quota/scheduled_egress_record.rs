use super::*;

use acme_kit::text;

/// Where a scheduled egress record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledEgressRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledEgressRecordStatus {
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

impl std::fmt::Display for ScheduledEgressRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled egress record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressRecord {
    pub code: String,
    pub label: String,
    pub title: String,
    pub country: String,
    pub approved: bool,
    pub cost: f64,
    pub category: String,
    pub ttl_seconds: i64,
    pub status: ScheduledEgressRecordStatus,
}

impl ScheduledEgressRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledEgressRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledEgressRecordStatus) -> bool {
        match self.status {
            ScheduledEgressRecordStatus::Open => {
                match next {
                    ScheduledEgressRecordStatus::InReview | ScheduledEgressRecordStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEgressRecordStatus::InReview => {
                return next == ScheduledEgressRecordStatus::Resolved;
            }
            ScheduledEgressRecordStatus::Resolved => {
                return next == ScheduledEgressRecordStatus::Archived;
            }
            ScheduledEgressRecordStatus::Reopened => {
                match next {
                    ScheduledEgressRecordStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledEgressRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled egress record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        (self.cost * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledEgressRecord {
        ScheduledEgressRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            approved: self.approved,
            cost: self.cost,
            category: self.category.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled egress record in status open.
pub fn new_scheduled_egress_record(code: &str, label: &str) -> ScheduledEgressRecord {
    ScheduledEgressRecord {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        country: "".to_string(),
        approved: false,
        cost: 0.0,
        category: "".to_string(),
        ttl_seconds: 0,
        status: ScheduledEgressRecordStatus::Open,
    }
}
