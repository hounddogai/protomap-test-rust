use super::*;

use acme_kit::text;

/// Where a adaptive action item score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveActionItemScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveActionItemScoreStatus {
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

impl std::fmt::Display for AdaptiveActionItemScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive action item score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveActionItemScore {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub attempts: i64,
    pub retries: i64,
    pub expires_at: i64,
    pub status: AdaptiveActionItemScoreStatus,
}

impl AdaptiveActionItemScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveActionItemScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveActionItemScoreStatus) -> bool {
        match self.status {
            AdaptiveActionItemScoreStatus::Open => {
                return next == AdaptiveActionItemScoreStatus::InReview
                    || next == AdaptiveActionItemScoreStatus::Resolved;
            }
            AdaptiveActionItemScoreStatus::InReview => {
                return next == AdaptiveActionItemScoreStatus::Resolved
                    || next == AdaptiveActionItemScoreStatus::Archived;
            }
            AdaptiveActionItemScoreStatus::Resolved => {
                match next {
                    AdaptiveActionItemScoreStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveActionItemScoreStatus::Reopened => {
                match next {
                    AdaptiveActionItemScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveActionItemScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive action item score {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveActionItemScore {
        AdaptiveActionItemScore {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            attempts: self.attempts,
            retries: self.retries,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a adaptive action item score in status open.
pub fn new_adaptive_action_item_score(code: &str, label: &str) -> AdaptiveActionItemScore {
    AdaptiveActionItemScore {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        attempts: 0,
        retries: 0,
        expires_at: 0,
        status: AdaptiveActionItemScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_action_item_score("code-537", "label-624");
        assert_eq!(item.status, AdaptiveActionItemScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_action_item_score("code-537", "label-624");
        if item.move_to(AdaptiveActionItemScoreStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveActionItemScoreStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_action_item_score("code-537", "label-624");
        item.status = AdaptiveActionItemScoreStatus::Archived;
        assert!(
            item.move_to(AdaptiveActionItemScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
