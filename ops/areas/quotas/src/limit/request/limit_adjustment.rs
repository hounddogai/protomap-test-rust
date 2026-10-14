use super::*;

use acme_kit::text;

/// Where a limit adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LimitAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LimitAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAdjustment {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub port: i64,
    pub currency: String,
    pub grace_days: i64,
    pub active: bool,
    pub rate: f64,
    pub status: LimitAdjustmentStatus,
}

impl LimitAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitAdjustmentStatus) -> bool {
        match self.status {
            LimitAdjustmentStatus::Planned => {
                match next {
                    LimitAdjustmentStatus::Completed | LimitAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitAdjustmentStatus::Scheduled => {
                return next == LimitAdjustmentStatus::InProgress || next == LimitAdjustmentStatus::Cancelled;
            }
            LimitAdjustmentStatus::InProgress => {
                return next == LimitAdjustmentStatus::Completed || next == LimitAdjustmentStatus::Cancelled;
            }
            LimitAdjustmentStatus::Completed => {
                return next == LimitAdjustmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "limit adjustment", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.rate + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitAdjustment {
        LimitAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            port: self.port,
            currency: self.currency.clone(),
            grace_days: self.grace_days,
            active: self.active,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a limit adjustment in status planned.
pub fn new_limit_adjustment(code: &str, label: &str) -> LimitAdjustment {
    LimitAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        port: 0,
        currency: "".to_string(),
        grace_days: 0,
        active: false,
        rate: 0.0,
        status: LimitAdjustmentStatus::Planned,
    }
}
