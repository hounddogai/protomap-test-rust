use super::*;

use acme_kit::text;

/// Where a scheduled action item rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledActionItemRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledActionItemRollupStatus {
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

impl std::fmt::Display for ScheduledActionItemRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled action item rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemRollup {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub cipher: String,
    pub summary: String,
    pub score: f64,
    pub status: ScheduledActionItemRollupStatus,
}

impl ScheduledActionItemRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledActionItemRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledActionItemRollupStatus) -> bool {
        match self.status {
            ScheduledActionItemRollupStatus::Open => {
                return next == ScheduledActionItemRollupStatus::InReview
                    || next == ScheduledActionItemRollupStatus::Archived;
            }
            ScheduledActionItemRollupStatus::InReview => {
                match next {
                    ScheduledActionItemRollupStatus::Resolved | ScheduledActionItemRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledActionItemRollupStatus::Resolved => {
                return next == ScheduledActionItemRollupStatus::Reopened
                    || next == ScheduledActionItemRollupStatus::Archived;
            }
            ScheduledActionItemRollupStatus::Reopened => {
                return next == ScheduledActionItemRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledActionItemRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled action item rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ScheduledActionItemRollup {
        ScheduledActionItemRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            cipher: self.cipher.clone(),
            summary: self.summary.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled action item rollup in status open.
pub fn new_scheduled_action_item_rollup(code: &str, label: &str) -> ScheduledActionItemRollup {
    ScheduledActionItemRollup {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        cipher: "".to_string(),
        summary: "".to_string(),
        score: 0.0,
        status: ScheduledActionItemRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_action_item_rollup("code-532", "label-838");
        assert_eq!(item.status, ScheduledActionItemRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_action_item_rollup("code-532", "label-838");
        if item.move_to(ScheduledActionItemRollupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledActionItemRollupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_action_item_rollup("code-532", "label-838");
        item.status = ScheduledActionItemRollupStatus::Archived;
        assert!(
            item.move_to(ScheduledActionItemRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
