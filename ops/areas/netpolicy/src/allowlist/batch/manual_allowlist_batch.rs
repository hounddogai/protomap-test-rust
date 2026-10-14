use super::*;

use acme_kit::text;

/// Where a manual allowlist batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAllowlistBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualAllowlistBatchStatus {
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

impl std::fmt::Display for ManualAllowlistBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual allowlist batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBatch {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub amount: f64,
    pub country: String,
    pub label: String,
    pub fingerprint: String,
    pub budget: f64,
    pub ttl_seconds: i64,
    pub scope: String,
    pub status: ManualAllowlistBatchStatus,
}

impl ManualAllowlistBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAllowlistBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAllowlistBatchStatus) -> bool {
        match self.status {
            ManualAllowlistBatchStatus::Open => {
                return next == ManualAllowlistBatchStatus::Resolved || next == ManualAllowlistBatchStatus::Archived;
            }
            ManualAllowlistBatchStatus::InReview => {
                match next {
                    ManualAllowlistBatchStatus::Resolved | ManualAllowlistBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAllowlistBatchStatus::Resolved => {
                match next {
                    ManualAllowlistBatchStatus::Reopened | ManualAllowlistBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAllowlistBatchStatus::Reopened => {
                match next {
                    ManualAllowlistBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: ManualAllowlistBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual allowlist batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.budget + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualAllowlistBatch {
        ManualAllowlistBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            amount: self.amount,
            country: self.country.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            budget: self.budget,
            ttl_seconds: self.ttl_seconds,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual allowlist batch in status open.
pub fn new_manual_allowlist_batch(code: &str, name: &str) -> ManualAllowlistBatch {
    ManualAllowlistBatch {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        amount: 0.0,
        country: "".to_string(),
        label: "".to_string(),
        fingerprint: "".to_string(),
        budget: 0.0,
        ttl_seconds: 0,
        scope: "".to_string(),
        status: ManualAllowlistBatchStatus::Open,
    }
}
