use super::*;

use acme_kit::text;

/// Where a rule item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleItemStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RuleItemStatus {
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

impl std::fmt::Display for RuleItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule item, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleItem {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub pinned: bool,
    pub active: bool,
    pub status: RuleItemStatus,
}

impl RuleItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleItemStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleItemStatus) -> bool {
        match self.status {
            RuleItemStatus::Open => {
                return next == RuleItemStatus::Resolved || next == RuleItemStatus::Archived;
            }
            RuleItemStatus::InReview => {
                return next == RuleItemStatus::Reopened || next == RuleItemStatus::Archived;
            }
            RuleItemStatus::Resolved => {
                return next == RuleItemStatus::Archived;
            }
            RuleItemStatus::Reopened => {
                return next == RuleItemStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> RuleItem {
        RuleItem {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            pinned: self.pinned,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rule item in status open.
pub fn new_rule_item(code: &str, title: &str) -> RuleItem {
    RuleItem {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        pinned: false,
        active: false,
        status: RuleItemStatus::Open,
    }
}
