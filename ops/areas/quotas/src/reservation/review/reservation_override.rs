use super::*;

use acme_kit::text;

/// Where a reservation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReservationOverrideStatus {
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

impl std::fmt::Display for ReservationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationOverride {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub approved: bool,
    pub retries: i64,
    pub namespace: String,
    pub status: ReservationOverrideStatus,
}

impl ReservationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationOverrideStatus) -> bool {
        match self.status {
            ReservationOverrideStatus::Open => {
                return next == ReservationOverrideStatus::Resolved;
            }
            ReservationOverrideStatus::InReview => {
                match next {
                    ReservationOverrideStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationOverrideStatus::Resolved => {
                match next {
                    ReservationOverrideStatus::Reopened | ReservationOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationOverrideStatus::Reopened => {
                match next {
                    ReservationOverrideStatus::Archived => {
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

    pub fn move_to(&mut self, next: ReservationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.attempts as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationOverride {
        ReservationOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            approved: self.approved,
            retries: self.retries,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a reservation override in status open.
pub fn new_reservation_override(code: &str, label: &str) -> ReservationOverride {
    ReservationOverride {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        approved: false,
        retries: 0,
        namespace: "".to_string(),
        status: ReservationOverrideStatus::Open,
    }
}
