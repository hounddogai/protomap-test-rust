use super::*;

use acme_kit::text;

/// Where a retention review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionReviewStatus {
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

impl std::fmt::Display for RetentionReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReview {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub discount: f64,
    pub phone: String,
    pub fingerprint: String,
    pub batch_size: i64,
    pub title: String,
    pub status: RetentionReviewStatus,
}

impl RetentionReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionReviewStatus) -> bool {
        match self.status {
            RetentionReviewStatus::New => {
                return next == RetentionReviewStatus::Done || next == RetentionReviewStatus::Failed;
            }
            RetentionReviewStatus::Queued => {
                return next == RetentionReviewStatus::Running || next == RetentionReviewStatus::Failed;
            }
            RetentionReviewStatus::Running => {
                match next {
                    RetentionReviewStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionReviewStatus::Done => {
                return next == RetentionReviewStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "retention review", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.batch_size as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionReview {
        RetentionReview {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            discount: self.discount,
            phone: self.phone.clone(),
            fingerprint: self.fingerprint.clone(),
            batch_size: self.batch_size,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention review in status new.
pub fn new_retention_review(code: &str, label: &str) -> RetentionReview {
    RetentionReview {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        discount: 0.0,
        phone: "".to_string(),
        fingerprint: "".to_string(),
        batch_size: 0,
        title: "".to_string(),
        status: RetentionReviewStatus::New,
    }
}
