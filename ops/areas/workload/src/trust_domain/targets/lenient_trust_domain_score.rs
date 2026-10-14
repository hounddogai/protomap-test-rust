use super::*;

use acme_kit::text;

/// Where a lenient trust domain score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTrustDomainScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientTrustDomainScoreStatus {
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

impl std::fmt::Display for LenientTrustDomainScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient trust domain score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainScore {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub score: f64,
    pub segment: String,
    pub cost: f64,
    pub archived: bool,
    pub budget: f64,
    pub principal: String,
    pub status: LenientTrustDomainScoreStatus,
}

impl LenientTrustDomainScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTrustDomainScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTrustDomainScoreStatus) -> bool {
        match self.status {
            LenientTrustDomainScoreStatus::Open => {
                return next == LenientTrustDomainScoreStatus::Resolved;
            }
            LenientTrustDomainScoreStatus::InReview => {
                match next {
                    LenientTrustDomainScoreStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTrustDomainScoreStatus::Resolved => {
                return next == LenientTrustDomainScoreStatus::Archived;
            }
            LenientTrustDomainScoreStatus::Reopened => {
                return next == LenientTrustDomainScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientTrustDomainScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient trust domain score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.cost + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTrustDomainScore {
        LenientTrustDomainScore {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            score: self.score,
            segment: self.segment.clone(),
            cost: self.cost,
            archived: self.archived,
            budget: self.budget,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a lenient trust domain score in status open.
pub fn new_lenient_trust_domain_score(code: &str, label: &str) -> LenientTrustDomainScore {
    LenientTrustDomainScore {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        score: 0.0,
        segment: "".to_string(),
        cost: 0.0,
        archived: false,
        budget: 0.0,
        principal: "".to_string(),
        status: LenientTrustDomainScoreStatus::Open,
    }
}
