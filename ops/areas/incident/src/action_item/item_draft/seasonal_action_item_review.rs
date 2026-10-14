use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal action item review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalActionItemReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalActionItemReviewStatus {
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

impl std::fmt::Display for SeasonalActionItemReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal action item review as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemReview {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub attributes: BTreeMap<String, String>,
    pub threshold: f64,
    pub status: SeasonalActionItemReviewStatus,
}

impl SeasonalActionItemReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalActionItemReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalActionItemReviewStatus) -> bool {
        match self.status {
            SeasonalActionItemReviewStatus::Open => {
                return next == SeasonalActionItemReviewStatus::Resolved;
            }
            SeasonalActionItemReviewStatus::InReview => {
                match next {
                    SeasonalActionItemReviewStatus::Resolved | SeasonalActionItemReviewStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalActionItemReviewStatus::Resolved => {
                match next {
                    SeasonalActionItemReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalActionItemReviewStatus::Reopened => {
                match next {
                    SeasonalActionItemReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalActionItemReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal action item review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> SeasonalActionItemReview {
        SeasonalActionItemReview {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            attributes: self.attributes.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal action item review in status open.
pub fn new_seasonal_action_item_review(code: &str, label: &str) -> SeasonalActionItemReview {
    SeasonalActionItemReview {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        attributes: BTreeMap::new(),
        threshold: 0.0,
        status: SeasonalActionItemReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_action_item_review("code-187", "label-445");
        assert_eq!(item.status, SeasonalActionItemReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_seasonal_action_item_review("code-187", "label-445");
        if item.move_to(SeasonalActionItemReviewStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalActionItemReviewStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_action_item_review("code-187", "label-445");
        item.status = SeasonalActionItemReviewStatus::Archived;
        assert!(
            item.move_to(SeasonalActionItemReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
