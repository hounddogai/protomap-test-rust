use super::*;

use acme_kit::text;

/// Where a action item target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ActionItemTargetStatus {
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

impl std::fmt::Display for ActionItemTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTarget {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub expires_at: i64,
    pub amount: f64,
    pub currency: String,
    pub segment: String,
    pub status: ActionItemTargetStatus,
}

impl ActionItemTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemTargetStatus) -> bool {
        match self.status {
            ActionItemTargetStatus::Open => {
                match next {
                    ActionItemTargetStatus::InReview | ActionItemTargetStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemTargetStatus::InReview => {
                return next == ActionItemTargetStatus::Resolved || next == ActionItemTargetStatus::Reopened;
            }
            ActionItemTargetStatus::Resolved => {
                match next {
                    ActionItemTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemTargetStatus::Reopened => {
                return next == ActionItemTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item target", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.expires_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemTarget {
        ActionItemTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            expires_at: self.expires_at,
            amount: self.amount,
            currency: self.currency.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a action item target in status open.
pub fn new_action_item_target(code: &str, name: &str) -> ActionItemTarget {
    ActionItemTarget {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        expires_at: 0,
        amount: 0.0,
        currency: "".to_string(),
        segment: "".to_string(),
        status: ActionItemTargetStatus::Open,
    }
}
