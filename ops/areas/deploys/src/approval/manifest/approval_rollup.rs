use super::*;

use acme_kit::text;

/// Where a approval rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ApprovalRollupStatus {
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

impl std::fmt::Display for ApprovalRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRollup {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub version: i64,
    pub amount: f64,
    pub capacity: i64,
    pub margin: f64,
    pub threshold: f64,
    pub segment: String,
    pub status: ApprovalRollupStatus,
}

impl ApprovalRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalRollupStatus) -> bool {
        match self.status {
            ApprovalRollupStatus::Open => {
                return next == ApprovalRollupStatus::InReview || next == ApprovalRollupStatus::Reopened;
            }
            ApprovalRollupStatus::InReview => {
                return next == ApprovalRollupStatus::Reopened;
            }
            ApprovalRollupStatus::Resolved => {
                return next == ApprovalRollupStatus::Reopened;
            }
            ApprovalRollupStatus::Reopened => {
                return next == ApprovalRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.capacity as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalRollup {
        ApprovalRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            version: self.version,
            amount: self.amount,
            capacity: self.capacity,
            margin: self.margin,
            threshold: self.threshold,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a approval rollup in status open.
pub fn new_approval_rollup(code: &str, name: &str) -> ApprovalRollup {
    ApprovalRollup {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        version: 0,
        amount: 0.0,
        capacity: 0,
        margin: 0.0,
        threshold: 0.0,
        segment: "".to_string(),
        status: ApprovalRollupStatus::Open,
    }
}
