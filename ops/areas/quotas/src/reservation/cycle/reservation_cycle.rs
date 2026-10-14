use super::*;

use acme_kit::text;

/// Where a reservation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReservationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReservationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationCycle {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub duration_minutes: i64,
    pub attempts: i64,
    pub version: i64,
    pub window_days: i64,
    pub ends_at: i64,
    pub batch_size: i64,
    pub status: ReservationCycleStatus,
}

impl ReservationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationCycleStatus) -> bool {
        match self.status {
            ReservationCycleStatus::Created => {
                match next {
                    ReservationCycleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationCycleStatus::Validated => {
                match next {
                    ReservationCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationCycleStatus::Published => {
                return next == ReservationCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "reservation cycle", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.version as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationCycle {
        ReservationCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            duration_minutes: self.duration_minutes,
            attempts: self.attempts,
            version: self.version,
            window_days: self.window_days,
            ends_at: self.ends_at,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a reservation cycle in status created.
pub fn new_reservation_cycle(code: &str, name: &str) -> ReservationCycle {
    ReservationCycle {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        duration_minutes: 0,
        attempts: 0,
        version: 0,
        window_days: 0,
        ends_at: 0,
        batch_size: 0,
        status: ReservationCycleStatus::Created,
    }
}
