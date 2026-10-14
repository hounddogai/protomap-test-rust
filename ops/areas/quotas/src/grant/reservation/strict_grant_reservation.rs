use super::*;

use acme_kit::text;

/// Where a strict grant reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictGrantReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictGrantReservationStatus {
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

impl std::fmt::Display for StrictGrantReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict grant reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantReservation {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub score: f64,
    pub created_at: i64,
    pub channel: String,
    pub rate: f64,
    pub status: StrictGrantReservationStatus,
}

impl StrictGrantReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictGrantReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictGrantReservationStatus) -> bool {
        match self.status {
            StrictGrantReservationStatus::Idle => {
                match next {
                    StrictGrantReservationStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictGrantReservationStatus::Warming => {
                return next == StrictGrantReservationStatus::Stopped;
            }
            StrictGrantReservationStatus::Ready => {
                return next == StrictGrantReservationStatus::Draining;
            }
            StrictGrantReservationStatus::Draining => {
                match next {
                    StrictGrantReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictGrantReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict grant reservation {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.rate - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictGrantReservation {
        StrictGrantReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            score: self.score,
            created_at: self.created_at,
            channel: self.channel.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict grant reservation in status idle.
pub fn new_strict_grant_reservation(code: &str, name: &str) -> StrictGrantReservation {
    StrictGrantReservation {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        score: 0.0,
        created_at: 0,
        channel: "".to_string(),
        rate: 0.0,
        status: StrictGrantReservationStatus::Idle,
    }
}
