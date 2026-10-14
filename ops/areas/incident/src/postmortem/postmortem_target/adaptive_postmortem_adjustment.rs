use super::*;

use acme_kit::text;

/// Where a adaptive postmortem adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePostmortemAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptivePostmortemAdjustmentStatus {
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

impl std::fmt::Display for AdaptivePostmortemAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive postmortem adjustment as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemAdjustment {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub ceiling: f64,
    pub attempts: i64,
    pub grace_days: i64,
    pub status: AdaptivePostmortemAdjustmentStatus,
}

impl AdaptivePostmortemAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePostmortemAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePostmortemAdjustmentStatus) -> bool {
        match self.status {
            AdaptivePostmortemAdjustmentStatus::Idle => {
                return next == AdaptivePostmortemAdjustmentStatus::Warming
                    || next == AdaptivePostmortemAdjustmentStatus::Draining;
            }
            AdaptivePostmortemAdjustmentStatus::Warming => {
                match next {
                    AdaptivePostmortemAdjustmentStatus::Ready | AdaptivePostmortemAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePostmortemAdjustmentStatus::Ready => {
                match next {
                    AdaptivePostmortemAdjustmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePostmortemAdjustmentStatus::Draining => {
                return next == AdaptivePostmortemAdjustmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptivePostmortemAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive postmortem adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.ceiling * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePostmortemAdjustment {
        AdaptivePostmortemAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            ceiling: self.ceiling,
            attempts: self.attempts,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive postmortem adjustment in status idle.
pub fn new_adaptive_postmortem_adjustment(code: &str, label: &str) -> AdaptivePostmortemAdjustment {
    AdaptivePostmortemAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        ceiling: 0.0,
        attempts: 0,
        grace_days: 0,
        status: AdaptivePostmortemAdjustmentStatus::Idle,
    }
}
