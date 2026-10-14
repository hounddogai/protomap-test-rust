use super::*;

use acme_kit::text;

/// Where a pending verifier review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingVerifierReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingVerifierReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PendingVerifierReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending verifier review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierReview {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub principal: String,
    pub ttl_seconds: i64,
    pub discount: f64,
    pub ratio: f64,
    pub unit: String,
    pub attempts: i64,
    pub limit: i64,
    pub status: PendingVerifierReviewStatus,
}

impl PendingVerifierReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingVerifierReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingVerifierReviewStatus) -> bool {
        match self.status {
            PendingVerifierReviewStatus::Draft => {
                match next {
                    PendingVerifierReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingVerifierReviewStatus::Pending => {
                return next == PendingVerifierReviewStatus::Suspended || next == PendingVerifierReviewStatus::Closed;
            }
            PendingVerifierReviewStatus::Active => {
                match next {
                    PendingVerifierReviewStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingVerifierReviewStatus::Suspended => {
                return next == PendingVerifierReviewStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingVerifierReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending verifier review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingVerifierReview {
        PendingVerifierReview {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            discount: self.discount,
            ratio: self.ratio,
            unit: self.unit.clone(),
            attempts: self.attempts,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending verifier review in status draft.
pub fn new_pending_verifier_review(code: &str, label: &str) -> PendingVerifierReview {
    PendingVerifierReview {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        principal: "".to_string(),
        ttl_seconds: 0,
        discount: 0.0,
        ratio: 0.0,
        unit: "".to_string(),
        attempts: 0,
        limit: 0,
        status: PendingVerifierReviewStatus::Draft,
    }
}
