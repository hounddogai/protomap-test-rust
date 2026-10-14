use super::*;

use acme_kit::text;

/// Where a weighted slo reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSloReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedSloReservationStatus {
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

impl std::fmt::Display for WeightedSloReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted slo reservation as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSloReservation {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub channel: String,
    pub window_days: i64,
    pub attempts: i64,
    pub ends_at: i64,
    pub timezone: String,
    pub burst: i64,
    pub updated_at: i64,
    pub status: WeightedSloReservationStatus,
}

impl WeightedSloReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSloReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSloReservationStatus) -> bool {
        match self.status {
            WeightedSloReservationStatus::New => {
                return next == WeightedSloReservationStatus::Done;
            }
            WeightedSloReservationStatus::Queued => {
                return next == WeightedSloReservationStatus::Running || next == WeightedSloReservationStatus::Done;
            }
            WeightedSloReservationStatus::Running => {
                return next == WeightedSloReservationStatus::Done || next == WeightedSloReservationStatus::Failed;
            }
            WeightedSloReservationStatus::Done => {
                return next == WeightedSloReservationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedSloReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted slo reservation {} cannot move from {} to {}",
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
        format!("{} {}", "weighted slo reservation", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.burst as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedSloReservation {
        WeightedSloReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            channel: self.channel.clone(),
            window_days: self.window_days,
            attempts: self.attempts,
            ends_at: self.ends_at,
            timezone: self.timezone.clone(),
            burst: self.burst,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a weighted slo reservation in status new.
pub fn new_weighted_slo_reservation(code: &str, label: &str) -> WeightedSloReservation {
    WeightedSloReservation {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        channel: "".to_string(),
        window_days: 0,
        attempts: 0,
        ends_at: 0,
        timezone: "".to_string(),
        burst: 0,
        updated_at: 0,
        status: WeightedSloReservationStatus::New,
    }
}
