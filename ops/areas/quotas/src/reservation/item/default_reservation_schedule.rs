use super::*;

use acme_kit::text;

/// Where a default reservation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReservationScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultReservationScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultReservationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default reservation schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSchedule {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub summary: String,
    pub priority: i64,
    pub unit: String,
    pub reviewer: String,
    pub rate: f64,
    pub score: f64,
    pub status: DefaultReservationScheduleStatus,
}

impl DefaultReservationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReservationScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReservationScheduleStatus) -> bool {
        match self.status {
            DefaultReservationScheduleStatus::Requested => {
                return next == DefaultReservationScheduleStatus::Revoked
                    || next == DefaultReservationScheduleStatus::Expired;
            }
            DefaultReservationScheduleStatus::Granted => {
                match next {
                    DefaultReservationScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReservationScheduleStatus::Revoked => {
                match next {
                    DefaultReservationScheduleStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultReservationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default reservation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default reservation schedule", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultReservationSchedule {
        DefaultReservationSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            summary: self.summary.clone(),
            priority: self.priority,
            unit: self.unit.clone(),
            reviewer: self.reviewer.clone(),
            rate: self.rate,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a default reservation schedule in status requested.
pub fn new_default_reservation_schedule(code: &str, title: &str) -> DefaultReservationSchedule {
    DefaultReservationSchedule {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        summary: "".to_string(),
        priority: 0,
        unit: "".to_string(),
        reviewer: "".to_string(),
        rate: 0.0,
        score: 0.0,
        status: DefaultReservationScheduleStatus::Requested,
    }
}
