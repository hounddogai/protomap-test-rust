use super::*;

use acme_kit::text;

/// Where a limit reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LimitReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LimitReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a limit reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitReservation {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub weight: f64,
    pub limit: i64,
    pub starts_at: i64,
    pub quota_units: i64,
    pub floor_value: f64,
    pub cluster: String,
    pub status: LimitReservationStatus,
}

impl LimitReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitReservationStatus) -> bool {
        match self.status {
            LimitReservationStatus::Created => {
                match next {
                    LimitReservationStatus::Validated | LimitReservationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitReservationStatus::Validated => {
                return next == LimitReservationStatus::Published;
            }
            LimitReservationStatus::Published => {
                match next {
                    LimitReservationStatus::Retired => {
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

    pub fn move_to(&mut self, next: LimitReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.floor_value + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitReservation {
        LimitReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            weight: self.weight,
            limit: self.limit,
            starts_at: self.starts_at,
            quota_units: self.quota_units,
            floor_value: self.floor_value,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a limit reservation in status created.
pub fn new_limit_reservation(code: &str, title: &str) -> LimitReservation {
    LimitReservation {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        weight: 0.0,
        limit: 0,
        starts_at: 0,
        quota_units: 0,
        floor_value: 0.0,
        cluster: "".to_string(),
        status: LimitReservationStatus::Created,
    }
}
