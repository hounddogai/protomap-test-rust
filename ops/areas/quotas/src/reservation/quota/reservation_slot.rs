use super::*;

use acme_kit::text;

/// Where a reservation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ReservationSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ReservationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSlot {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub rotation_days: i64,
    pub margin: f64,
    pub notes: String,
    pub floor_value: f64,
    pub timezone: String,
    pub cost: f64,
    pub archived: bool,
    pub status: ReservationSlotStatus,
}

impl ReservationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationSlotStatus) -> bool {
        match self.status {
            ReservationSlotStatus::New => {
                match next {
                    ReservationSlotStatus::Queued | ReservationSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationSlotStatus::Queued => {
                return next == ReservationSlotStatus::Running || next == ReservationSlotStatus::Failed;
            }
            ReservationSlotStatus::Running => {
                match next {
                    ReservationSlotStatus::Done | ReservationSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationSlotStatus::Done => {
                return next == ReservationSlotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.margin * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationSlot {
        ReservationSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            rotation_days: self.rotation_days,
            margin: self.margin,
            notes: self.notes.clone(),
            floor_value: self.floor_value,
            timezone: self.timezone.clone(),
            cost: self.cost,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a reservation slot in status new.
pub fn new_reservation_slot(code: &str, name: &str) -> ReservationSlot {
    ReservationSlot {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        rotation_days: 0,
        margin: 0.0,
        notes: "".to_string(),
        floor_value: 0.0,
        timezone: "".to_string(),
        cost: 0.0,
        archived: false,
        status: ReservationSlotStatus::New,
    }
}
