use super::*;

use acme_kit::text;

/// Where a batched action item draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedActionItemDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedActionItemDraftStatus {
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

impl std::fmt::Display for BatchedActionItemDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched action item draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedActionItemDraft {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub approved: bool,
    pub maximum: f64,
    pub active: bool,
    pub description: String,
    pub margin: f64,
    pub rotation_days: i64,
    pub status: BatchedActionItemDraftStatus,
}

impl BatchedActionItemDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedActionItemDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedActionItemDraftStatus) -> bool {
        match self.status {
            BatchedActionItemDraftStatus::Planned => {
                match next {
                    BatchedActionItemDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedActionItemDraftStatus::Scheduled => {
                return next == BatchedActionItemDraftStatus::Cancelled;
            }
            BatchedActionItemDraftStatus::InProgress => {
                return next == BatchedActionItemDraftStatus::Completed
                    || next == BatchedActionItemDraftStatus::Cancelled;
            }
            BatchedActionItemDraftStatus::Completed => {
                return next == BatchedActionItemDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedActionItemDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched action item draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.maximum - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedActionItemDraft {
        BatchedActionItemDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            approved: self.approved,
            maximum: self.maximum,
            active: self.active,
            description: self.description.clone(),
            margin: self.margin,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched action item draft in status planned.
pub fn new_batched_action_item_draft(code: &str, title: &str) -> BatchedActionItemDraft {
    BatchedActionItemDraft {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        approved: false,
        maximum: 0.0,
        active: false,
        description: "".to_string(),
        margin: 0.0,
        rotation_days: 0,
        status: BatchedActionItemDraftStatus::Planned,
    }
}
