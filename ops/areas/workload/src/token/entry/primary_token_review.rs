use super::*;

use acme_kit::text;

/// Where a primary token review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTokenReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryTokenReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryTokenReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary token review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTokenReview {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub retries: i64,
    pub floor_value: f64,
    pub category: String,
    pub status: PrimaryTokenReviewStatus,
}

impl PrimaryTokenReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTokenReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTokenReviewStatus) -> bool {
        match self.status {
            PrimaryTokenReviewStatus::Requested => {
                return next == PrimaryTokenReviewStatus::Granted || next == PrimaryTokenReviewStatus::Expired;
            }
            PrimaryTokenReviewStatus::Granted => {
                match next {
                    PrimaryTokenReviewStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTokenReviewStatus::Revoked => {
                return next == PrimaryTokenReviewStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryTokenReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary token review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary token review", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTokenReview {
        PrimaryTokenReview {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            retries: self.retries,
            floor_value: self.floor_value,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary token review in status requested.
pub fn new_primary_token_review(code: &str, label: &str) -> PrimaryTokenReview {
    PrimaryTokenReview {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        retries: 0,
        floor_value: 0.0,
        category: "".to_string(),
        status: PrimaryTokenReviewStatus::Requested,
    }
}
