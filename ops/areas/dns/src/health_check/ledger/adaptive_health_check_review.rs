use super::*;

use acme_kit::text;

/// Where a adaptive health check review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveHealthCheckReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveHealthCheckReviewStatus {
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

impl std::fmt::Display for AdaptiveHealthCheckReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive health check review as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckReview {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub retries: i64,
    pub principal: String,
    pub quantity: i64,
    pub scope: String,
    pub title: String,
    pub cluster: String,
    pub status: AdaptiveHealthCheckReviewStatus,
}

impl AdaptiveHealthCheckReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveHealthCheckReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveHealthCheckReviewStatus) -> bool {
        match self.status {
            AdaptiveHealthCheckReviewStatus::Open => {
                return next == AdaptiveHealthCheckReviewStatus::InReview
                    || next == AdaptiveHealthCheckReviewStatus::Reopened;
            }
            AdaptiveHealthCheckReviewStatus::InReview => {
                return next == AdaptiveHealthCheckReviewStatus::Reopened
                    || next == AdaptiveHealthCheckReviewStatus::Archived;
            }
            AdaptiveHealthCheckReviewStatus::Resolved => {
                match next {
                    AdaptiveHealthCheckReviewStatus::Reopened | AdaptiveHealthCheckReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveHealthCheckReviewStatus::Reopened => {
                match next {
                    AdaptiveHealthCheckReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveHealthCheckReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive health check review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quantity as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveHealthCheckReview {
        AdaptiveHealthCheckReview {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            retries: self.retries,
            principal: self.principal.clone(),
            quantity: self.quantity,
            scope: self.scope.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive health check review in status open.
pub fn new_adaptive_health_check_review(code: &str, label: &str) -> AdaptiveHealthCheckReview {
    AdaptiveHealthCheckReview {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        retries: 0,
        principal: "".to_string(),
        quantity: 0,
        scope: "".to_string(),
        title: "".to_string(),
        cluster: "".to_string(),
        status: AdaptiveHealthCheckReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_health_check_review("code-621", "label-147");
        assert_eq!(item.status, AdaptiveHealthCheckReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_health_check_review("code-621", "label-147");
        if item.move_to(AdaptiveHealthCheckReviewStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveHealthCheckReviewStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_health_check_review("code-621", "label-147");
        item.status = AdaptiveHealthCheckReviewStatus::Archived;
        assert!(
            item.move_to(AdaptiveHealthCheckReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
