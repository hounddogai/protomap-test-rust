use super::*;

use acme_kit::text;

/// Where a action item policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ActionItemPolicyStatus {
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

impl std::fmt::Display for ActionItemPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item policy as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPolicy {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub tags: Vec<String>,
    pub budget: f64,
    pub burst: i64,
    pub duration_minutes: i64,
    pub status: ActionItemPolicyStatus,
}

impl ActionItemPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemPolicyStatus) -> bool {
        match self.status {
            ActionItemPolicyStatus::Open => {
                return next == ActionItemPolicyStatus::Resolved;
            }
            ActionItemPolicyStatus::InReview => {
                return next == ActionItemPolicyStatus::Resolved || next == ActionItemPolicyStatus::Archived;
            }
            ActionItemPolicyStatus::Resolved => {
                return next == ActionItemPolicyStatus::Reopened || next == ActionItemPolicyStatus::Archived;
            }
            ActionItemPolicyStatus::Reopened => {
                match next {
                    ActionItemPolicyStatus::Archived => {
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

    pub fn move_to(&mut self, next: ActionItemPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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
        ((self.duration_minutes as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemPolicy {
        ActionItemPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            tags: self.tags.clone(),
            budget: self.budget,
            burst: self.burst,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a action item policy in status open.
pub fn new_action_item_policy(code: &str, name: &str) -> ActionItemPolicy {
    ActionItemPolicy {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        tags: Vec::new(),
        budget: 0.0,
        burst: 0,
        duration_minutes: 0,
        status: ActionItemPolicyStatus::Open,
    }
}
