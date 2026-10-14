use super::*;

use acme_kit::text;

/// Where a action item slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ActionItemSlotStatus {
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

impl std::fmt::Display for ActionItemSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a action item slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSlot {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub ttl_seconds: i64,
    pub limit: i64,
    pub category: String,
    pub ratio: f64,
    pub tags: Vec<String>,
    pub floor_value: f64,
    pub summary: String,
    pub maximum: f64,
    pub status: ActionItemSlotStatus,
}

impl ActionItemSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemSlotStatus) -> bool {
        match self.status {
            ActionItemSlotStatus::New => {
                return next == ActionItemSlotStatus::Queued || next == ActionItemSlotStatus::Failed;
            }
            ActionItemSlotStatus::Queued => {
                match next {
                    ActionItemSlotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemSlotStatus::Running => {
                match next {
                    ActionItemSlotStatus::Done | ActionItemSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemSlotStatus::Done => {
                return next == ActionItemSlotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item slot", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.limit as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemSlot {
        ActionItemSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            ttl_seconds: self.ttl_seconds,
            limit: self.limit,
            category: self.category.clone(),
            ratio: self.ratio,
            tags: self.tags.clone(),
            floor_value: self.floor_value,
            summary: self.summary.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a action item slot in status new.
pub fn new_action_item_slot(code: &str, title: &str) -> ActionItemSlot {
    ActionItemSlot {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        ttl_seconds: 0,
        limit: 0,
        category: "".to_string(),
        ratio: 0.0,
        tags: Vec::new(),
        floor_value: 0.0,
        summary: "".to_string(),
        maximum: 0.0,
        status: ActionItemSlotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_action_item_slot("code-718", "title-612");
        assert_eq!(item.status, ActionItemSlotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_action_item_slot("code-718", "title-612");
        if item.move_to(ActionItemSlotStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemSlotStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_action_item_slot("code-718", "title-612");
        item.status = ActionItemSlotStatus::Failed;
        assert!(
            item.move_to(ActionItemSlotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
