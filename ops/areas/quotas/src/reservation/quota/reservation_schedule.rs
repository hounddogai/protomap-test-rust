use super::*;

use acme_kit::text;

/// Where a reservation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationScheduleStatus {
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

impl std::fmt::Display for ReservationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation schedule as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSchedule {
    pub code: String,
    pub title: String,
    pub description: String,
    pub created_at: i64,
    pub grace_days: i64,
    pub status: ReservationScheduleStatus,
}

impl ReservationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationScheduleStatus) -> bool {
        match self.status {
            ReservationScheduleStatus::Idle => {
                return next == ReservationScheduleStatus::Draining;
            }
            ReservationScheduleStatus::Warming => {
                return next == ReservationScheduleStatus::Draining;
            }
            ReservationScheduleStatus::Ready => {
                match next {
                    ReservationScheduleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationScheduleStatus::Draining => {
                return next == ReservationScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "reservation schedule", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.created_at as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationSchedule {
        ReservationSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            created_at: self.created_at,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a reservation schedule in status idle.
pub fn new_reservation_schedule(code: &str, title: &str) -> ReservationSchedule {
    ReservationSchedule {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        created_at: 0,
        grace_days: 0,
        status: ReservationScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_reservation_schedule("code-236", "title-497");
        assert_eq!(item.status, ReservationScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_reservation_schedule("code-236", "title-497");
        if item.move_to(ReservationScheduleStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationScheduleStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_reservation_schedule("code-236", "title-497");
        item.status = ReservationScheduleStatus::Stopped;
        assert!(
            item.move_to(ReservationScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
