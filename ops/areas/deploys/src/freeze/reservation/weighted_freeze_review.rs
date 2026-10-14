use super::*;

use acme_kit::text;

/// Where a weighted freeze review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFreezeReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedFreezeReviewStatus {
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

impl std::fmt::Display for WeightedFreezeReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted freeze review as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReview {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub port: i64,
    pub pinned: bool,
    pub status: WeightedFreezeReviewStatus,
}

impl WeightedFreezeReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFreezeReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFreezeReviewStatus) -> bool {
        match self.status {
            WeightedFreezeReviewStatus::Open => {
                return next == WeightedFreezeReviewStatus::Archived;
            }
            WeightedFreezeReviewStatus::InReview => {
                match next {
                    WeightedFreezeReviewStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFreezeReviewStatus::Resolved => {
                return next == WeightedFreezeReviewStatus::Reopened || next == WeightedFreezeReviewStatus::Archived;
            }
            WeightedFreezeReviewStatus::Reopened => {
                match next {
                    WeightedFreezeReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: WeightedFreezeReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted freeze review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> WeightedFreezeReview {
        WeightedFreezeReview {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            port: self.port,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted freeze review in status open.
pub fn new_weighted_freeze_review(code: &str, label: &str) -> WeightedFreezeReview {
    WeightedFreezeReview {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        port: 0,
        pinned: false,
        status: WeightedFreezeReviewStatus::Open,
    }
}
