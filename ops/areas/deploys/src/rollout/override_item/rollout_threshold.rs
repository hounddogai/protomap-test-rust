use super::*;

use acme_kit::text;

/// Where a rollout threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RolloutThresholdStatus {
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

impl std::fmt::Display for RolloutThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutThreshold {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub notes: String,
    pub country: String,
    pub ratio: f64,
    pub batch_size: i64,
    pub cipher: String,
    pub status: RolloutThresholdStatus,
}

impl RolloutThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutThresholdStatus) -> bool {
        match self.status {
            RolloutThresholdStatus::Open => {
                return next == RolloutThresholdStatus::Reopened || next == RolloutThresholdStatus::Archived;
            }
            RolloutThresholdStatus::InReview => {
                return next == RolloutThresholdStatus::Resolved;
            }
            RolloutThresholdStatus::Resolved => {
                match next {
                    RolloutThresholdStatus::Reopened | RolloutThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutThresholdStatus::Reopened => {
                return next == RolloutThresholdStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout threshold", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.batch_size as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutThreshold {
        RolloutThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            notes: self.notes.clone(),
            country: self.country.clone(),
            ratio: self.ratio,
            batch_size: self.batch_size,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rollout threshold in status open.
pub fn new_rollout_threshold(code: &str, name: &str) -> RolloutThreshold {
    RolloutThreshold {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        notes: "".to_string(),
        country: "".to_string(),
        ratio: 0.0,
        batch_size: 0,
        cipher: "".to_string(),
        status: RolloutThresholdStatus::Open,
    }
}
