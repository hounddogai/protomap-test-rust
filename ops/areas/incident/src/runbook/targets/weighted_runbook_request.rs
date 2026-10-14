use super::*;

use acme_kit::text;

/// Where a weighted runbook request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRunbookRequestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedRunbookRequestStatus {
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

impl std::fmt::Display for WeightedRunbookRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted runbook request as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookRequest {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub cipher: String,
    pub threshold: f64,
    pub source: String,
    pub status: WeightedRunbookRequestStatus,
}

impl WeightedRunbookRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRunbookRequestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRunbookRequestStatus) -> bool {
        match self.status {
            WeightedRunbookRequestStatus::Open => {
                return next == WeightedRunbookRequestStatus::Reopened
                    || next == WeightedRunbookRequestStatus::Archived;
            }
            WeightedRunbookRequestStatus::InReview => {
                match next {
                    WeightedRunbookRequestStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRunbookRequestStatus::Resolved => {
                match next {
                    WeightedRunbookRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRunbookRequestStatus::Reopened => {
                return next == WeightedRunbookRequestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRunbookRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted runbook request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.threshold * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRunbookRequest {
        WeightedRunbookRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            cipher: self.cipher.clone(),
            threshold: self.threshold,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted runbook request in status open.
pub fn new_weighted_runbook_request(code: &str, label: &str) -> WeightedRunbookRequest {
    WeightedRunbookRequest {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        cipher: "".to_string(),
        threshold: 0.0,
        source: "".to_string(),
        status: WeightedRunbookRequestStatus::Open,
    }
}
