use super::*;

use acme_kit::text;

/// Where a regional approval reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalApprovalReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalApprovalReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalApprovalReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional approval reservation as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalReservation {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub locale: String,
    pub ratio: f64,
    pub revision: i64,
    pub timezone: String,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub status: RegionalApprovalReservationStatus,
}

impl RegionalApprovalReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalApprovalReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalApprovalReservationStatus) -> bool {
        match self.status {
            RegionalApprovalReservationStatus::Created => {
                return next == RegionalApprovalReservationStatus::Validated
                    || next == RegionalApprovalReservationStatus::Retired;
            }
            RegionalApprovalReservationStatus::Validated => {
                match next {
                    RegionalApprovalReservationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalApprovalReservationStatus::Published => {
                return next == RegionalApprovalReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalApprovalReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional approval reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.revision as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalApprovalReservation {
        RegionalApprovalReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            ratio: self.ratio,
            revision: self.revision,
            timezone: self.timezone.clone(),
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional approval reservation in status created.
pub fn new_regional_approval_reservation(code: &str, title: &str) -> RegionalApprovalReservation {
    RegionalApprovalReservation {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        locale: "".to_string(),
        ratio: 0.0,
        revision: 0,
        timezone: "".to_string(),
        ttl_seconds: 0,
        expires_at: 0,
        status: RegionalApprovalReservationStatus::Created,
    }
}
