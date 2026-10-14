use super::*;

use acme_kit::text;

/// Where a default retention checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRetentionCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultRetentionCheckpointStatus {
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

impl std::fmt::Display for DefaultRetentionCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default retention checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionCheckpoint {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub category: String,
    pub quantity: i64,
    pub fingerprint: String,
    pub revision: i64,
    pub limit: i64,
    pub budget: f64,
    pub status: DefaultRetentionCheckpointStatus,
}

impl DefaultRetentionCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRetentionCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRetentionCheckpointStatus) -> bool {
        match self.status {
            DefaultRetentionCheckpointStatus::Open => {
                match next {
                    DefaultRetentionCheckpointStatus::Resolved | DefaultRetentionCheckpointStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRetentionCheckpointStatus::InReview => {
                match next {
                    DefaultRetentionCheckpointStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRetentionCheckpointStatus::Resolved => {
                return next == DefaultRetentionCheckpointStatus::Reopened
                    || next == DefaultRetentionCheckpointStatus::Archived;
            }
            DefaultRetentionCheckpointStatus::Reopened => {
                match next {
                    DefaultRetentionCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultRetentionCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default retention checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.limit as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRetentionCheckpoint {
        DefaultRetentionCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            category: self.category.clone(),
            quantity: self.quantity,
            fingerprint: self.fingerprint.clone(),
            revision: self.revision,
            limit: self.limit,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default retention checkpoint in status open.
pub fn new_default_retention_checkpoint(code: &str, title: &str) -> DefaultRetentionCheckpoint {
    DefaultRetentionCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        category: "".to_string(),
        quantity: 0,
        fingerprint: "".to_string(),
        revision: 0,
        limit: 0,
        budget: 0.0,
        status: DefaultRetentionCheckpointStatus::Open,
    }
}
