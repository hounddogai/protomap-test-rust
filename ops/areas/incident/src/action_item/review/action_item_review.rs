use super::*;

use acme_kit::text;

/// Where a action item review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemReviewStatus {
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

impl std::fmt::Display for ActionItemReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item review as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemReview {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub cost: f64,
    pub currency: String,
    pub duration_minutes: i64,
    pub status: ActionItemReviewStatus,
}

impl ActionItemReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemReviewStatus) -> bool {
        match self.status {
            ActionItemReviewStatus::Idle => {
                match next {
                    ActionItemReviewStatus::Warming | ActionItemReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemReviewStatus::Warming => {
                match next {
                    ActionItemReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemReviewStatus::Ready => {
                match next {
                    ActionItemReviewStatus::Draining | ActionItemReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemReviewStatus::Draining => {
                return next == ActionItemReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item review {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.cost - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemReview {
        ActionItemReview {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            cost: self.cost,
            currency: self.currency.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a action item review in status idle.
pub fn new_action_item_review(code: &str, label: &str) -> ActionItemReview {
    ActionItemReview {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        cost: 0.0,
        currency: "".to_string(),
        duration_minutes: 0,
        status: ActionItemReviewStatus::Idle,
    }
}
