use super::*;

use acme_kit::text;

/// Where a reservation entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ReservationEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation entry as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationEntry {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub pinned: bool,
    pub ends_at: i64,
    pub status: ReservationEntryStatus,
}

impl ReservationEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationEntryStatus) -> bool {
        match self.status {
            ReservationEntryStatus::Idle => {
                return next == ReservationEntryStatus::Warming || next == ReservationEntryStatus::Draining;
            }
            ReservationEntryStatus::Warming => {
                return next == ReservationEntryStatus::Draining || next == ReservationEntryStatus::Stopped;
            }
            ReservationEntryStatus::Ready => {
                match next {
                    ReservationEntryStatus::Draining | ReservationEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationEntryStatus::Draining => {
                return next == ReservationEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationEntry {
        ReservationEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            pinned: self.pinned,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a reservation entry in status idle.
pub fn new_reservation_entry(code: &str, label: &str) -> ReservationEntry {
    ReservationEntry {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        pinned: false,
        ends_at: 0,
        status: ReservationEntryStatus::Idle,
    }
}
