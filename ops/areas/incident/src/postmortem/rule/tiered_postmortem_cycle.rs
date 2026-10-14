use super::*;

use acme_kit::text;

/// Where a tiered postmortem cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPostmortemCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredPostmortemCycleStatus {
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

impl std::fmt::Display for TieredPostmortemCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered postmortem cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemCycle {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub duration_minutes: i64,
    pub host: String,
    pub floor_value: f64,
    pub description: String,
    pub burst: i64,
    pub status: TieredPostmortemCycleStatus,
}

impl TieredPostmortemCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPostmortemCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPostmortemCycleStatus) -> bool {
        match self.status {
            TieredPostmortemCycleStatus::Idle => {
                return next == TieredPostmortemCycleStatus::Warming || next == TieredPostmortemCycleStatus::Stopped;
            }
            TieredPostmortemCycleStatus::Warming => {
                match next {
                    TieredPostmortemCycleStatus::Ready | TieredPostmortemCycleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPostmortemCycleStatus::Ready => {
                match next {
                    TieredPostmortemCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPostmortemCycleStatus::Draining => {
                return next == TieredPostmortemCycleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredPostmortemCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered postmortem cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPostmortemCycle {
        TieredPostmortemCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            duration_minutes: self.duration_minutes,
            host: self.host.clone(),
            floor_value: self.floor_value,
            description: self.description.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered postmortem cycle in status idle.
pub fn new_tiered_postmortem_cycle(code: &str, label: &str) -> TieredPostmortemCycle {
    TieredPostmortemCycle {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        duration_minutes: 0,
        host: "".to_string(),
        floor_value: 0.0,
        description: "".to_string(),
        burst: 0,
        status: TieredPostmortemCycleStatus::Idle,
    }
}
