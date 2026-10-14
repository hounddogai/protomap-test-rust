use super::*;

use acme_kit::text;

/// Where a rolling reservation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingReservationExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingReservationExceptionStatus {
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

impl std::fmt::Display for RollingReservationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling reservation exception as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationException {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub locale: String,
    pub audience: String,
    pub version: i64,
    pub status: RollingReservationExceptionStatus,
}

impl RollingReservationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingReservationExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingReservationExceptionStatus) -> bool {
        match self.status {
            RollingReservationExceptionStatus::Open => {
                return next == RollingReservationExceptionStatus::Archived;
            }
            RollingReservationExceptionStatus::InReview => {
                return next == RollingReservationExceptionStatus::Reopened;
            }
            RollingReservationExceptionStatus::Resolved => {
                match next {
                    RollingReservationExceptionStatus::Reopened | RollingReservationExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingReservationExceptionStatus::Reopened => {
                return next == RollingReservationExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingReservationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling reservation exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> RollingReservationException {
        RollingReservationException {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            locale: self.locale.clone(),
            audience: self.audience.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rolling reservation exception in status open.
pub fn new_rolling_reservation_exception(code: &str, label: &str) -> RollingReservationException {
    RollingReservationException {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        locale: "".to_string(),
        audience: "".to_string(),
        version: 0,
        status: RollingReservationExceptionStatus::Open,
    }
}
