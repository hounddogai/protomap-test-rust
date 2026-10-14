use super::*;

use acme_kit::text;

/// Where a adaptive identity schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveIdentityScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveIdentityScheduleStatus {
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

impl std::fmt::Display for AdaptiveIdentityScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive identity schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySchedule {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub issuer: String,
    pub sequence: i64,
    pub phone: String,
    pub port: i64,
    pub grace_days: i64,
    pub revision: i64,
    pub weight: f64,
    pub status: AdaptiveIdentityScheduleStatus,
}

impl AdaptiveIdentitySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveIdentityScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveIdentityScheduleStatus) -> bool {
        match self.status {
            AdaptiveIdentityScheduleStatus::Open => {
                return next == AdaptiveIdentityScheduleStatus::InReview
                    || next == AdaptiveIdentityScheduleStatus::Resolved;
            }
            AdaptiveIdentityScheduleStatus::InReview => {
                return next == AdaptiveIdentityScheduleStatus::Resolved
                    || next == AdaptiveIdentityScheduleStatus::Reopened;
            }
            AdaptiveIdentityScheduleStatus::Resolved => {
                return next == AdaptiveIdentityScheduleStatus::Reopened
                    || next == AdaptiveIdentityScheduleStatus::Archived;
            }
            AdaptiveIdentityScheduleStatus::Reopened => {
                return next == AdaptiveIdentityScheduleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveIdentityScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive identity schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive identity schedule", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.weight - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveIdentitySchedule {
        AdaptiveIdentitySchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            issuer: self.issuer.clone(),
            sequence: self.sequence,
            phone: self.phone.clone(),
            port: self.port,
            grace_days: self.grace_days,
            revision: self.revision,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive identity schedule in status open.
pub fn new_adaptive_identity_schedule(code: &str, title: &str) -> AdaptiveIdentitySchedule {
    AdaptiveIdentitySchedule {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        issuer: "".to_string(),
        sequence: 0,
        phone: "".to_string(),
        port: 0,
        grace_days: 0,
        revision: 0,
        weight: 0.0,
        status: AdaptiveIdentityScheduleStatus::Open,
    }
}
