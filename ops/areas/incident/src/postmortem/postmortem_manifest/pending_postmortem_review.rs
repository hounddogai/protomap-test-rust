use super::*;

use acme_kit::text;

/// Where a pending postmortem review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPostmortemReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingPostmortemReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingPostmortemReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending postmortem review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemReview {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub priority: i64,
    pub currency: String,
    pub status: PendingPostmortemReviewStatus,
}

impl PendingPostmortemReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPostmortemReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPostmortemReviewStatus) -> bool {
        match self.status {
            PendingPostmortemReviewStatus::Requested => {
                return next == PendingPostmortemReviewStatus::Expired;
            }
            PendingPostmortemReviewStatus::Granted => {
                return next == PendingPostmortemReviewStatus::Revoked
                    || next == PendingPostmortemReviewStatus::Expired;
            }
            PendingPostmortemReviewStatus::Revoked => {
                match next {
                    PendingPostmortemReviewStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingPostmortemReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending postmortem review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending postmortem review", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPostmortemReview {
        PendingPostmortemReview {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            priority: self.priority,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending postmortem review in status requested.
pub fn new_pending_postmortem_review(code: &str, title: &str) -> PendingPostmortemReview {
    PendingPostmortemReview {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        external_ref: "".to_string(),
        priority: 0,
        currency: "".to_string(),
        status: PendingPostmortemReviewStatus::Requested,
    }
}
