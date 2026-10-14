use super::*;

use acme_kit::text;

/// Where a lenient measurement notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientMeasurementNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientMeasurementNoticeStatus {
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

impl std::fmt::Display for LenientMeasurementNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient measurement notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementNotice {
    pub code: String,
    pub title: String,
    pub name: String,
    pub owner_id: String,
    pub quantity: i64,
    pub floor_value: f64,
    pub status: LenientMeasurementNoticeStatus,
}

impl LenientMeasurementNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientMeasurementNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientMeasurementNoticeStatus) -> bool {
        match self.status {
            LenientMeasurementNoticeStatus::Idle => {
                return next == LenientMeasurementNoticeStatus::Draining;
            }
            LenientMeasurementNoticeStatus::Warming => {
                return next == LenientMeasurementNoticeStatus::Stopped;
            }
            LenientMeasurementNoticeStatus::Ready => {
                return next == LenientMeasurementNoticeStatus::Draining
                    || next == LenientMeasurementNoticeStatus::Stopped;
            }
            LenientMeasurementNoticeStatus::Draining => {
                match next {
                    LenientMeasurementNoticeStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientMeasurementNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient measurement notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient measurement notice", self.code)
    }

    pub fn net_value(&self) -> f64 {
        (self.floor_value * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientMeasurementNotice {
        LenientMeasurementNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            quantity: self.quantity,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient measurement notice in status idle.
pub fn new_lenient_measurement_notice(code: &str, title: &str) -> LenientMeasurementNotice {
    LenientMeasurementNotice {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        owner_id: "".to_string(),
        quantity: 0,
        floor_value: 0.0,
        status: LenientMeasurementNoticeStatus::Idle,
    }
}
