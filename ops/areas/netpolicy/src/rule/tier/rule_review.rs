use super::*;

use acme_kit::text;

/// Where a rule review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RuleReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RuleReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleReview {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub rate: f64,
    pub checksum: String,
    pub floor_value: f64,
    pub fingerprint: String,
    pub cipher: String,
    pub ttl_seconds: i64,
    pub audience: String,
    pub summary: String,
    pub status: RuleReviewStatus,
}

impl RuleReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleReviewStatus) -> bool {
        match self.status {
            RuleReviewStatus::New => {
                match next {
                    RuleReviewStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleReviewStatus::Queued => {
                return next == RuleReviewStatus::Running || next == RuleReviewStatus::Done;
            }
            RuleReviewStatus::Running => {
                match next {
                    RuleReviewStatus::Done | RuleReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleReviewStatus::Done => {
                return next == RuleReviewStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule review", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.rate - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleReview {
        RuleReview {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            rate: self.rate,
            checksum: self.checksum.clone(),
            floor_value: self.floor_value,
            fingerprint: self.fingerprint.clone(),
            cipher: self.cipher.clone(),
            ttl_seconds: self.ttl_seconds,
            audience: self.audience.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rule review in status new.
pub fn new_rule_review(code: &str, name: &str) -> RuleReview {
    RuleReview {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        rate: 0.0,
        checksum: "".to_string(),
        floor_value: 0.0,
        fingerprint: "".to_string(),
        cipher: "".to_string(),
        ttl_seconds: 0,
        audience: "".to_string(),
        summary: "".to_string(),
        status: RuleReviewStatus::New,
    }
}
