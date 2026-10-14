use super::*;

use acme_kit::text;

/// Where a weighted trust domain review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTrustDomainReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedTrustDomainReviewStatus {
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

impl std::fmt::Display for WeightedTrustDomainReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted trust domain review as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainReview {
    pub code: String,
    pub title: String,
    pub region: String,
    pub sequence: i64,
    pub description: String,
    pub tenant_id: String,
    pub limit: i64,
    pub attempts: i64,
    pub audience: String,
    pub cipher: String,
    pub status: WeightedTrustDomainReviewStatus,
}

impl WeightedTrustDomainReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTrustDomainReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTrustDomainReviewStatus) -> bool {
        match self.status {
            WeightedTrustDomainReviewStatus::New => {
                match next {
                    WeightedTrustDomainReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTrustDomainReviewStatus::Queued => {
                return next == WeightedTrustDomainReviewStatus::Running
                    || next == WeightedTrustDomainReviewStatus::Failed;
            }
            WeightedTrustDomainReviewStatus::Running => {
                return next == WeightedTrustDomainReviewStatus::Done
                    || next == WeightedTrustDomainReviewStatus::Failed;
            }
            WeightedTrustDomainReviewStatus::Done => {
                return next == WeightedTrustDomainReviewStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTrustDomainReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted trust domain review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.limit as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTrustDomainReview {
        WeightedTrustDomainReview {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            sequence: self.sequence,
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            limit: self.limit,
            attempts: self.attempts,
            audience: self.audience.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a weighted trust domain review in status new.
pub fn new_weighted_trust_domain_review(code: &str, title: &str) -> WeightedTrustDomainReview {
    WeightedTrustDomainReview {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        sequence: 0,
        description: "".to_string(),
        tenant_id: "".to_string(),
        limit: 0,
        attempts: 0,
        audience: "".to_string(),
        cipher: "".to_string(),
        status: WeightedTrustDomainReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_trust_domain_review("code-805", "title-639");
        assert_eq!(item.status, WeightedTrustDomainReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_weighted_trust_domain_review("code-805", "title-639");
        if item.move_to(WeightedTrustDomainReviewStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTrustDomainReviewStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_trust_domain_review("code-805", "title-639");
        item.status = WeightedTrustDomainReviewStatus::Failed;
        assert!(
            item.move_to(WeightedTrustDomainReviewStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
