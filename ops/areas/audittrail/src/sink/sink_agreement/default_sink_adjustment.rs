use super::*;

use acme_kit::text;

/// Where a default sink adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSinkAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultSinkAdjustmentStatus {
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

impl std::fmt::Display for DefaultSinkAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default sink adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkAdjustment {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub owner_id: String,
    pub segment: String,
    pub amount: f64,
    pub status: DefaultSinkAdjustmentStatus,
}

impl DefaultSinkAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSinkAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSinkAdjustmentStatus) -> bool {
        match self.status {
            DefaultSinkAdjustmentStatus::New => {
                return next == DefaultSinkAdjustmentStatus::Failed;
            }
            DefaultSinkAdjustmentStatus::Queued => {
                match next {
                    DefaultSinkAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSinkAdjustmentStatus::Running => {
                return next == DefaultSinkAdjustmentStatus::Failed;
            }
            DefaultSinkAdjustmentStatus::Done => {
                return next == DefaultSinkAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultSinkAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default sink adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default sink adjustment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> DefaultSinkAdjustment {
        DefaultSinkAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            owner_id: self.owner_id.clone(),
            segment: self.segment.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a default sink adjustment in status new.
pub fn new_default_sink_adjustment(code: &str, title: &str) -> DefaultSinkAdjustment {
    DefaultSinkAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        owner_id: "".to_string(),
        segment: "".to_string(),
        amount: 0.0,
        status: DefaultSinkAdjustmentStatus::New,
    }
}
