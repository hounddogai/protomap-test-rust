use super::*;

use acme_kit::text;

/// Where a segment forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SegmentForecastStatus {
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

impl std::fmt::Display for SegmentForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecast {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub key_id: String,
    pub audience: String,
    pub status: SegmentForecastStatus,
}

impl SegmentForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentForecastStatus) -> bool {
        match self.status {
            SegmentForecastStatus::Idle => {
                match next {
                    SegmentForecastStatus::Ready | SegmentForecastStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentForecastStatus::Warming => {
                return next == SegmentForecastStatus::Draining || next == SegmentForecastStatus::Stopped;
            }
            SegmentForecastStatus::Ready => {
                return next == SegmentForecastStatus::Draining;
            }
            SegmentForecastStatus::Draining => {
                match next {
                    SegmentForecastStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SegmentForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> SegmentForecast {
        SegmentForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            key_id: self.key_id.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a segment forecast in status idle.
pub fn new_segment_forecast(code: &str, label: &str) -> SegmentForecast {
    SegmentForecast {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        key_id: "".to_string(),
        audience: "".to_string(),
        status: SegmentForecastStatus::Idle,
    }
}
