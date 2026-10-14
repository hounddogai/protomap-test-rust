use super::*;

use acme_kit::text;

/// Where a primary action item batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryActionItemBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryActionItemBatchStatus {
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

impl std::fmt::Display for PrimaryActionItemBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary action item batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBatch {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub limit: i64,
    pub archived: bool,
    pub priority: i64,
    pub discount: f64,
    pub status: PrimaryActionItemBatchStatus,
}

impl PrimaryActionItemBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryActionItemBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryActionItemBatchStatus) -> bool {
        match self.status {
            PrimaryActionItemBatchStatus::Idle => {
                return next == PrimaryActionItemBatchStatus::Stopped;
            }
            PrimaryActionItemBatchStatus::Warming => {
                return next == PrimaryActionItemBatchStatus::Ready || next == PrimaryActionItemBatchStatus::Draining;
            }
            PrimaryActionItemBatchStatus::Ready => {
                match next {
                    PrimaryActionItemBatchStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryActionItemBatchStatus::Draining => {
                return next == PrimaryActionItemBatchStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryActionItemBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary action item batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.discount + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryActionItemBatch {
        PrimaryActionItemBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            archived: self.archived,
            priority: self.priority,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary action item batch in status idle.
pub fn new_primary_action_item_batch(code: &str, label: &str) -> PrimaryActionItemBatch {
    PrimaryActionItemBatch {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        limit: 0,
        archived: false,
        priority: 0,
        discount: 0.0,
        status: PrimaryActionItemBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_action_item_batch("code-344", "label-559");
        assert_eq!(item.status, PrimaryActionItemBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_primary_action_item_batch("code-344", "label-559");
        if item.move_to(PrimaryActionItemBatchStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryActionItemBatchStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_action_item_batch("code-344", "label-559");
        item.status = PrimaryActionItemBatchStatus::Stopped;
        assert!(
            item.move_to(PrimaryActionItemBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
