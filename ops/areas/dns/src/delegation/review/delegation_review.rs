use super::*;

use acme_kit::text;

/// Where a delegation review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DelegationReviewStatus {
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

impl std::fmt::Display for DelegationReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationReview {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub discount: f64,
    pub phone: String,
    pub quota_units: i64,
    pub status: DelegationReviewStatus,
}

impl DelegationReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationReviewStatus) -> bool {
        match self.status {
            DelegationReviewStatus::New => {
                match next {
                    DelegationReviewStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationReviewStatus::Queued => {
                return next == DelegationReviewStatus::Running || next == DelegationReviewStatus::Done;
            }
            DelegationReviewStatus::Running => {
                match next {
                    DelegationReviewStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationReviewStatus::Done => {
                return next == DelegationReviewStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quota_units as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationReview {
        DelegationReview {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            discount: self.discount,
            phone: self.phone.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation review in status new.
pub fn new_delegation_review(code: &str, label: &str) -> DelegationReview {
    DelegationReview {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        discount: 0.0,
        phone: "".to_string(),
        quota_units: 0,
        status: DelegationReviewStatus::New,
    }
}
