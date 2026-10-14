use super::*;

use acme_kit::text;

/// Where a action item notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ActionItemNoticeStatus {
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

impl std::fmt::Display for ActionItemNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a action item notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemNotice {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub attempts: i64,
    pub ends_at: i64,
    pub key_id: String,
    pub status: ActionItemNoticeStatus,
}

impl ActionItemNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemNoticeStatus) -> bool {
        match self.status {
            ActionItemNoticeStatus::Open => {
                return next == ActionItemNoticeStatus::Reopened || next == ActionItemNoticeStatus::Archived;
            }
            ActionItemNoticeStatus::InReview => {
                return next == ActionItemNoticeStatus::Reopened || next == ActionItemNoticeStatus::Archived;
            }
            ActionItemNoticeStatus::Resolved => {
                return next == ActionItemNoticeStatus::Reopened || next == ActionItemNoticeStatus::Archived;
            }
            ActionItemNoticeStatus::Reopened => {
                match next {
                    ActionItemNoticeStatus::Archived => {
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

    pub fn move_to(&mut self, next: ActionItemNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ends_at as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemNotice {
        ActionItemNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            attempts: self.attempts,
            ends_at: self.ends_at,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item notice in status open.
pub fn new_action_item_notice(code: &str, name: &str) -> ActionItemNotice {
    ActionItemNotice {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        attempts: 0,
        ends_at: 0,
        key_id: "".to_string(),
        status: ActionItemNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_action_item_notice("code-576", "name-614");
        assert_eq!(item.status, ActionItemNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_action_item_notice("code-576", "name-614");
        if item.move_to(ActionItemNoticeStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemNoticeStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_action_item_notice("code-576", "name-614");
        item.status = ActionItemNoticeStatus::Archived;
        assert!(
            item.move_to(ActionItemNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
