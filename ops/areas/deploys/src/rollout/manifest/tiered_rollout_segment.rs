use super::*;

use acme_kit::text;

/// Where a tiered rollout segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRolloutSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredRolloutSegmentStatus {
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

impl std::fmt::Display for TieredRolloutSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered rollout segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSegment {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub batch_size: i64,
    pub audience: String,
    pub status: TieredRolloutSegmentStatus,
}

impl TieredRolloutSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRolloutSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRolloutSegmentStatus) -> bool {
        match self.status {
            TieredRolloutSegmentStatus::Idle => {
                match next {
                    TieredRolloutSegmentStatus::Ready | TieredRolloutSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRolloutSegmentStatus::Warming => {
                match next {
                    TieredRolloutSegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRolloutSegmentStatus::Ready => {
                match next {
                    TieredRolloutSegmentStatus::Draining | TieredRolloutSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRolloutSegmentStatus::Draining => {
                match next {
                    TieredRolloutSegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredRolloutSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered rollout segment {} cannot move from {} to {}",
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
        self.batch_size as f64
    }

    pub fn copy(&self) -> TieredRolloutSegment {
        TieredRolloutSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            batch_size: self.batch_size,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered rollout segment in status idle.
pub fn new_tiered_rollout_segment(code: &str, label: &str) -> TieredRolloutSegment {
    TieredRolloutSegment {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        batch_size: 0,
        audience: "".to_string(),
        status: TieredRolloutSegmentStatus::Idle,
    }
}
