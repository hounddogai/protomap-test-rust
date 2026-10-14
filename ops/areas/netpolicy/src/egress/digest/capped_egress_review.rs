use super::*;

use acme_kit::text;

/// Where a capped egress review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEgressReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedEgressReviewStatus {
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

impl std::fmt::Display for CappedEgressReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped egress review as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressReview {
    pub code: String,
    pub title: String,
    pub host: String,
    pub pinned: bool,
    pub quota_units: i64,
    pub segment: String,
    pub score: f64,
    pub currency: String,
    pub unit: String,
    pub status: CappedEgressReviewStatus,
}

impl CappedEgressReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEgressReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEgressReviewStatus) -> bool {
        match self.status {
            CappedEgressReviewStatus::Open => {
                return next == CappedEgressReviewStatus::Reopened;
            }
            CappedEgressReviewStatus::InReview => {
                return next == CappedEgressReviewStatus::Resolved || next == CappedEgressReviewStatus::Archived;
            }
            CappedEgressReviewStatus::Resolved => {
                return next == CappedEgressReviewStatus::Reopened || next == CappedEgressReviewStatus::Archived;
            }
            CappedEgressReviewStatus::Reopened => {
                match next {
                    CappedEgressReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: CappedEgressReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped egress review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quota_units as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedEgressReview {
        CappedEgressReview {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            pinned: self.pinned,
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            score: self.score,
            currency: self.currency.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a capped egress review in status open.
pub fn new_capped_egress_review(code: &str, title: &str) -> CappedEgressReview {
    CappedEgressReview {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        pinned: false,
        quota_units: 0,
        segment: "".to_string(),
        score: 0.0,
        currency: "".to_string(),
        unit: "".to_string(),
        status: CappedEgressReviewStatus::Open,
    }
}
