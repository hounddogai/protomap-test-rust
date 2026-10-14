use super::*;

use acme_kit::text;

/// Where a scheduled reservation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledReservationOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledReservationOverrideStatus {
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

impl std::fmt::Display for ScheduledReservationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled reservation override as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationOverride {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub name: String,
    pub locale: String,
    pub status: ScheduledReservationOverrideStatus,
}

impl ScheduledReservationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledReservationOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledReservationOverrideStatus) -> bool {
        match self.status {
            ScheduledReservationOverrideStatus::Open => {
                return next == ScheduledReservationOverrideStatus::InReview
                    || next == ScheduledReservationOverrideStatus::Reopened;
            }
            ScheduledReservationOverrideStatus::InReview => {
                return next == ScheduledReservationOverrideStatus::Reopened
                    || next == ScheduledReservationOverrideStatus::Archived;
            }
            ScheduledReservationOverrideStatus::Resolved => {
                return next == ScheduledReservationOverrideStatus::Reopened
                    || next == ScheduledReservationOverrideStatus::Archived;
            }
            ScheduledReservationOverrideStatus::Reopened => {
                return next == ScheduledReservationOverrideStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledReservationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled reservation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled reservation override", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> ScheduledReservationOverride {
        ScheduledReservationOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            name: self.name.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled reservation override in status open.
pub fn new_scheduled_reservation_override(code: &str, title: &str) -> ScheduledReservationOverride {
    ScheduledReservationOverride {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        name: "".to_string(),
        locale: "".to_string(),
        status: ScheduledReservationOverrideStatus::Open,
    }
}
