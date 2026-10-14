use super::*;

use acme_kit::text;

/// Where a strict reservation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictReservationSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictReservationSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictReservationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict reservation segment as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationSegment {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub weight: f64,
    pub rotation_days: i64,
    pub title: String,
    pub retries: i64,
    pub phone: String,
    pub cluster: String,
    pub attempts: i64,
    pub status: StrictReservationSegmentStatus,
}

impl StrictReservationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictReservationSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictReservationSegmentStatus) -> bool {
        match self.status {
            StrictReservationSegmentStatus::Draft => {
                return next == StrictReservationSegmentStatus::Pending
                    || next == StrictReservationSegmentStatus::Suspended;
            }
            StrictReservationSegmentStatus::Pending => {
                match next {
                    StrictReservationSegmentStatus::Active | StrictReservationSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReservationSegmentStatus::Active => {
                return next == StrictReservationSegmentStatus::Suspended
                    || next == StrictReservationSegmentStatus::Closed;
            }
            StrictReservationSegmentStatus::Suspended => {
                match next {
                    StrictReservationSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictReservationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict reservation segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.attempts as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictReservationSegment {
        StrictReservationSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            weight: self.weight,
            rotation_days: self.rotation_days,
            title: self.title.clone(),
            retries: self.retries,
            phone: self.phone.clone(),
            cluster: self.cluster.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a strict reservation segment in status draft.
pub fn new_strict_reservation_segment(code: &str, name: &str) -> StrictReservationSegment {
    StrictReservationSegment {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        weight: 0.0,
        rotation_days: 0,
        title: "".to_string(),
        retries: 0,
        phone: "".to_string(),
        cluster: "".to_string(),
        attempts: 0,
        status: StrictReservationSegmentStatus::Draft,
    }
}
