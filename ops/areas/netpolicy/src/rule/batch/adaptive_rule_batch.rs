use super::*;

use acme_kit::text;

/// Where a adaptive rule batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRuleBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveRuleBatchStatus {
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

impl std::fmt::Display for AdaptiveRuleBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive rule batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleBatch {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub revision: i64,
    pub channel: String,
    pub tags: Vec<String>,
    pub updated_at: i64,
    pub description: String,
    pub issuer: String,
    pub quota_units: i64,
    pub status: AdaptiveRuleBatchStatus,
}

impl AdaptiveRuleBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRuleBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRuleBatchStatus) -> bool {
        match self.status {
            AdaptiveRuleBatchStatus::Open => {
                return next == AdaptiveRuleBatchStatus::InReview || next == AdaptiveRuleBatchStatus::Archived;
            }
            AdaptiveRuleBatchStatus::InReview => {
                match next {
                    AdaptiveRuleBatchStatus::Resolved | AdaptiveRuleBatchStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRuleBatchStatus::Resolved => {
                return next == AdaptiveRuleBatchStatus::Reopened || next == AdaptiveRuleBatchStatus::Archived;
            }
            AdaptiveRuleBatchStatus::Reopened => {
                return next == AdaptiveRuleBatchStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRuleBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rule batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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
        ((self.expires_at as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRuleBatch {
        AdaptiveRuleBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            revision: self.revision,
            channel: self.channel.clone(),
            tags: self.tags.clone(),
            updated_at: self.updated_at,
            description: self.description.clone(),
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive rule batch in status open.
pub fn new_adaptive_rule_batch(code: &str, label: &str) -> AdaptiveRuleBatch {
    AdaptiveRuleBatch {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        revision: 0,
        channel: "".to_string(),
        tags: Vec::new(),
        updated_at: 0,
        description: "".to_string(),
        issuer: "".to_string(),
        quota_units: 0,
        status: AdaptiveRuleBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_rule_batch("code-799", "label-215");
        assert_eq!(item.status, AdaptiveRuleBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_rule_batch("code-799", "label-215");
        if item.move_to(AdaptiveRuleBatchStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRuleBatchStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_rule_batch("code-799", "label-215");
        item.status = AdaptiveRuleBatchStatus::Archived;
        assert!(
            item.move_to(AdaptiveRuleBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
