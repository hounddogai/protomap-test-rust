use super::*;

use acme_kit::text;

/// Where a fallback zone segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackZoneSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackZoneSegmentStatus {
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

impl std::fmt::Display for FallbackZoneSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback zone segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSegment {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub window_days: i64,
    pub visible: bool,
    pub status: FallbackZoneSegmentStatus,
}

impl FallbackZoneSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackZoneSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackZoneSegmentStatus) -> bool {
        match self.status {
            FallbackZoneSegmentStatus::New => {
                return next == FallbackZoneSegmentStatus::Running;
            }
            FallbackZoneSegmentStatus::Queued => {
                return next == FallbackZoneSegmentStatus::Done || next == FallbackZoneSegmentStatus::Failed;
            }
            FallbackZoneSegmentStatus::Running => {
                match next {
                    FallbackZoneSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackZoneSegmentStatus::Done => {
                match next {
                    FallbackZoneSegmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackZoneSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback zone segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.window_days as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackZoneSegment {
        FallbackZoneSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            window_days: self.window_days,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback zone segment in status new.
pub fn new_fallback_zone_segment(code: &str, title: &str) -> FallbackZoneSegment {
    FallbackZoneSegment {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        window_days: 0,
        visible: false,
        status: FallbackZoneSegmentStatus::New,
    }
}
