use super::*;

use acme_kit::text;

/// Where a freeze review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FreezeReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FreezeReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze review as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReview {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub batch_size: i64,
    pub issuer: String,
    pub grace_days: i64,
    pub approved: bool,
    pub status: FreezeReviewStatus,
}

impl FreezeReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeReviewStatus) -> bool {
        match self.status {
            FreezeReviewStatus::Proposed => {
                return next == FreezeReviewStatus::Rejected;
            }
            FreezeReviewStatus::Approved => {
                match next {
                    FreezeReviewStatus::Rejected | FreezeReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeReviewStatus::Rejected => {
                return next == FreezeReviewStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeReview {
        FreezeReview {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            grace_days: self.grace_days,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze review in status proposed.
pub fn new_freeze_review(code: &str, title: &str) -> FreezeReview {
    FreezeReview {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        batch_size: 0,
        issuer: "".to_string(),
        grace_days: 0,
        approved: false,
        status: FreezeReviewStatus::Proposed,
    }
}
