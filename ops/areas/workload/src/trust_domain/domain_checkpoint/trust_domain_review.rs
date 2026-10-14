use super::*;

use acme_kit::text;

/// Where a trust domain review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TrustDomainReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TrustDomainReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReview {
    pub code: String,
    pub title: String,
    pub description: String,
    pub segment: String,
    pub limit: i64,
    pub status: TrustDomainReviewStatus,
}

impl TrustDomainReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainReviewStatus) -> bool {
        match self.status {
            TrustDomainReviewStatus::Proposed => {
                match next {
                    TrustDomainReviewStatus::Rejected | TrustDomainReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainReviewStatus::Approved => {
                return next == TrustDomainReviewStatus::Rejected || next == TrustDomainReviewStatus::Withdrawn;
            }
            TrustDomainReviewStatus::Rejected => {
                match next {
                    TrustDomainReviewStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TrustDomainReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> TrustDomainReview {
        TrustDomainReview {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            segment: self.segment.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a trust domain review in status proposed.
pub fn new_trust_domain_review(code: &str, title: &str) -> TrustDomainReview {
    TrustDomainReview {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        segment: "".to_string(),
        limit: 0,
        status: TrustDomainReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_trust_domain_review("code-989", "title-891");
        assert_eq!(item.status, TrustDomainReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_trust_domain_review("code-989", "title-891");
        if item.move_to(TrustDomainReviewStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainReviewStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_trust_domain_review("code-989", "title-891");
        item.status = TrustDomainReviewStatus::Withdrawn;
        assert!(
            item.move_to(TrustDomainReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
