use super::*;

use acme_kit::text;

/// Where a action item batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ActionItemBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ActionItemBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item batch as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBatch {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub starts_at: i64,
    pub attempts: i64,
    pub quantity: i64,
    pub status: ActionItemBatchStatus,
}

impl ActionItemBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemBatchStatus) -> bool {
        match self.status {
            ActionItemBatchStatus::Open => {
                return next == ActionItemBatchStatus::InReview || next == ActionItemBatchStatus::Resolved;
            }
            ActionItemBatchStatus::InReview => {
                return next == ActionItemBatchStatus::Resolved;
            }
            ActionItemBatchStatus::Resolved => {
                match next {
                    ActionItemBatchStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemBatchStatus::Reopened => {
                return next == ActionItemBatchStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.starts_at as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemBatch {
        ActionItemBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            starts_at: self.starts_at,
            attempts: self.attempts,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item batch in status open.
pub fn new_action_item_batch(code: &str, name: &str) -> ActionItemBatch {
    ActionItemBatch {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        starts_at: 0,
        attempts: 0,
        quantity: 0,
        status: ActionItemBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_action_item_batch("code-564", "name-860");
        assert_eq!(item.status, ActionItemBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_action_item_batch("code-564", "name-860");
        if item.move_to(ActionItemBatchStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemBatchStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_action_item_batch("code-564", "name-860");
        item.status = ActionItemBatchStatus::Archived;
        assert!(
            item.move_to(ActionItemBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
