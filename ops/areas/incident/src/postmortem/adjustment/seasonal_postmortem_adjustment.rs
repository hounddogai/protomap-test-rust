use super::*;

use acme_kit::text;

/// Where a seasonal postmortem adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPostmortemAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalPostmortemAdjustmentStatus {
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

impl std::fmt::Display for SeasonalPostmortemAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal postmortem adjustment as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemAdjustment {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub notes: String,
    pub active: bool,
    pub priority: i64,
    pub host: String,
    pub status: SeasonalPostmortemAdjustmentStatus,
}

impl SeasonalPostmortemAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPostmortemAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPostmortemAdjustmentStatus) -> bool {
        match self.status {
            SeasonalPostmortemAdjustmentStatus::Planned => {
                match next {
                    SeasonalPostmortemAdjustmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPostmortemAdjustmentStatus::Scheduled => {
                return next == SeasonalPostmortemAdjustmentStatus::InProgress;
            }
            SeasonalPostmortemAdjustmentStatus::InProgress => {
                match next {
                    SeasonalPostmortemAdjustmentStatus::Completed | SeasonalPostmortemAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPostmortemAdjustmentStatus::Completed => {
                return next == SeasonalPostmortemAdjustmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalPostmortemAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal postmortem adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.priority as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalPostmortemAdjustment {
        SeasonalPostmortemAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            notes: self.notes.clone(),
            active: self.active,
            priority: self.priority,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal postmortem adjustment in status planned.
pub fn new_seasonal_postmortem_adjustment(code: &str, label: &str) -> SeasonalPostmortemAdjustment {
    SeasonalPostmortemAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        notes: "".to_string(),
        active: false,
        priority: 0,
        host: "".to_string(),
        status: SeasonalPostmortemAdjustmentStatus::Planned,
    }
}
