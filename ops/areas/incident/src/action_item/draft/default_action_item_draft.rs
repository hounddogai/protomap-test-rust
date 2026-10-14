use super::*;

use acme_kit::text;

/// Where a default action item draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultActionItemDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultActionItemDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultActionItemDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default action item draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemDraft {
    pub code: String,
    pub label: String,
    pub source: String,
    pub priority: i64,
    pub notes: String,
    pub status: DefaultActionItemDraftStatus,
}

impl DefaultActionItemDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultActionItemDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultActionItemDraftStatus) -> bool {
        match self.status {
            DefaultActionItemDraftStatus::Requested => {
                return next == DefaultActionItemDraftStatus::Expired;
            }
            DefaultActionItemDraftStatus::Granted => {
                match next {
                    DefaultActionItemDraftStatus::Revoked | DefaultActionItemDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultActionItemDraftStatus::Revoked => {
                match next {
                    DefaultActionItemDraftStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultActionItemDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default action item draft {} cannot move from {} to {}",
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
        self.priority as f64
    }

    pub fn copy(&self) -> DefaultActionItemDraft {
        DefaultActionItemDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            priority: self.priority,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default action item draft in status requested.
pub fn new_default_action_item_draft(code: &str, label: &str) -> DefaultActionItemDraft {
    DefaultActionItemDraft {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        priority: 0,
        notes: "".to_string(),
        status: DefaultActionItemDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_action_item_draft("code-778", "label-740");
        assert_eq!(item.status, DefaultActionItemDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_default_action_item_draft("code-778", "label-740");
        if item.move_to(DefaultActionItemDraftStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultActionItemDraftStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_action_item_draft("code-778", "label-740");
        item.status = DefaultActionItemDraftStatus::Expired;
        assert!(
            item.move_to(DefaultActionItemDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
