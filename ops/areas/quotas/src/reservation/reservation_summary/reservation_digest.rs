use super::*;

use acme_kit::text;

/// Where a reservation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ReservationDigestStatus {
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

impl std::fmt::Display for ReservationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDigest {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub issuer: String,
    pub channel: String,
    pub grace_days: i64,
    pub batch_size: i64,
    pub timezone: String,
    pub status: ReservationDigestStatus,
}

impl ReservationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationDigestStatus) -> bool {
        match self.status {
            ReservationDigestStatus::Planned => {
                match next {
                    ReservationDigestStatus::InProgress | ReservationDigestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationDigestStatus::Scheduled => {
                return next == ReservationDigestStatus::Completed;
            }
            ReservationDigestStatus::InProgress => {
                return next == ReservationDigestStatus::Cancelled;
            }
            ReservationDigestStatus::Completed => {
                match next {
                    ReservationDigestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ReservationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "reservation digest", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationDigest {
        ReservationDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            issuer: self.issuer.clone(),
            channel: self.channel.clone(),
            grace_days: self.grace_days,
            batch_size: self.batch_size,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a reservation digest in status planned.
pub fn new_reservation_digest(code: &str, label: &str) -> ReservationDigest {
    ReservationDigest {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        issuer: "".to_string(),
        channel: "".to_string(),
        grace_days: 0,
        batch_size: 0,
        timezone: "".to_string(),
        status: ReservationDigestStatus::Planned,
    }
}
