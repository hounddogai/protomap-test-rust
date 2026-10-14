use super::*;

use acme_kit::text;

/// Where a manual rule review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRuleReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualRuleReviewStatus {
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

impl std::fmt::Display for ManualRuleReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual rule review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleReview {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub revision: i64,
    pub country: String,
    pub created_at: i64,
    pub region: String,
    pub status: ManualRuleReviewStatus,
}

impl ManualRuleReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRuleReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRuleReviewStatus) -> bool {
        match self.status {
            ManualRuleReviewStatus::Idle => {
                return next == ManualRuleReviewStatus::Draining;
            }
            ManualRuleReviewStatus::Warming => {
                match next {
                    ManualRuleReviewStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRuleReviewStatus::Ready => {
                return next == ManualRuleReviewStatus::Draining || next == ManualRuleReviewStatus::Stopped;
            }
            ManualRuleReviewStatus::Draining => {
                match next {
                    ManualRuleReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualRuleReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rule review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRuleReview {
        ManualRuleReview {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            revision: self.revision,
            country: self.country.clone(),
            created_at: self.created_at,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual rule review in status idle.
pub fn new_manual_rule_review(code: &str, label: &str) -> ManualRuleReview {
    ManualRuleReview {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        revision: 0,
        country: "".to_string(),
        created_at: 0,
        region: "".to_string(),
        status: ManualRuleReviewStatus::Idle,
    }
}
