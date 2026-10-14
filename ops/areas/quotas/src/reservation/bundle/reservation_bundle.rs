use super::*;

use acme_kit::text;

/// Where a reservation bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationBundleStatus {
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

impl std::fmt::Display for ReservationBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBundle {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub version: i64,
    pub cipher: String,
    pub maximum: f64,
    pub ratio: f64,
    pub notes: String,
    pub key_id: String,
    pub priority: i64,
    pub title: String,
    pub status: ReservationBundleStatus,
}

impl ReservationBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationBundleStatus) -> bool {
        match self.status {
            ReservationBundleStatus::Idle => {
                return next == ReservationBundleStatus::Warming || next == ReservationBundleStatus::Stopped;
            }
            ReservationBundleStatus::Warming => {
                match next {
                    ReservationBundleStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationBundleStatus::Ready => {
                return next == ReservationBundleStatus::Draining;
            }
            ReservationBundleStatus::Draining => {
                return next == ReservationBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.version as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationBundle {
        ReservationBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            version: self.version,
            cipher: self.cipher.clone(),
            maximum: self.maximum,
            ratio: self.ratio,
            notes: self.notes.clone(),
            key_id: self.key_id.clone(),
            priority: self.priority,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a reservation bundle in status idle.
pub fn new_reservation_bundle(code: &str, label: &str) -> ReservationBundle {
    ReservationBundle {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        version: 0,
        cipher: "".to_string(),
        maximum: 0.0,
        ratio: 0.0,
        notes: "".to_string(),
        key_id: "".to_string(),
        priority: 0,
        title: "".to_string(),
        status: ReservationBundleStatus::Idle,
    }
}
