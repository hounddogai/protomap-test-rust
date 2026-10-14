use super::*;

use acme_kit::text;

/// Where a action item estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ActionItemEstimateStatus {
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

impl std::fmt::Display for ActionItemEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEstimate {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub unit: String,
    pub starts_at: i64,
    pub maximum: f64,
    pub archived: bool,
    pub rotation_days: i64,
    pub category: String,
    pub margin: f64,
    pub status: ActionItemEstimateStatus,
}

impl ActionItemEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemEstimateStatus) -> bool {
        match self.status {
            ActionItemEstimateStatus::Open => {
                return next == ActionItemEstimateStatus::Reopened || next == ActionItemEstimateStatus::Archived;
            }
            ActionItemEstimateStatus::InReview => {
                match next {
                    ActionItemEstimateStatus::Resolved | ActionItemEstimateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemEstimateStatus::Resolved => {
                return next == ActionItemEstimateStatus::Archived;
            }
            ActionItemEstimateStatus::Reopened => {
                match next {
                    ActionItemEstimateStatus::Archived => {
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

    pub fn move_to(&mut self, next: ActionItemEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item estimate {} cannot move from {} to {}",
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
        format!("{} {}", "action item estimate", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.margin + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemEstimate {
        ActionItemEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            unit: self.unit.clone(),
            starts_at: self.starts_at,
            maximum: self.maximum,
            archived: self.archived,
            rotation_days: self.rotation_days,
            category: self.category.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a action item estimate in status open.
pub fn new_action_item_estimate(code: &str, title: &str) -> ActionItemEstimate {
    ActionItemEstimate {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        unit: "".to_string(),
        starts_at: 0,
        maximum: 0.0,
        archived: false,
        rotation_days: 0,
        category: "".to_string(),
        margin: 0.0,
        status: ActionItemEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_action_item_estimate("code-594", "title-445");
        assert_eq!(item.status, ActionItemEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_action_item_estimate("code-594", "title-445");
        if item.move_to(ActionItemEstimateStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemEstimateStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_action_item_estimate("code-594", "title-445");
        item.status = ActionItemEstimateStatus::Archived;
        assert!(
            item.move_to(ActionItemEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
