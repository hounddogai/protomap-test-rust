use super::*;

use acme_kit::text;

/// Where a lenient approval exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientApprovalExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientApprovalExceptionStatus {
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

impl std::fmt::Display for LenientApprovalExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient approval exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalException {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub version: i64,
    pub cost: f64,
    pub maximum: f64,
    pub created_at: i64,
    pub attempts: i64,
    pub status: LenientApprovalExceptionStatus,
}

impl LenientApprovalException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientApprovalExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientApprovalExceptionStatus) -> bool {
        match self.status {
            LenientApprovalExceptionStatus::Open => {
                match next {
                    LenientApprovalExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientApprovalExceptionStatus::InReview => {
                match next {
                    LenientApprovalExceptionStatus::Resolved | LenientApprovalExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientApprovalExceptionStatus::Resolved => {
                return next == LenientApprovalExceptionStatus::Archived;
            }
            LenientApprovalExceptionStatus::Reopened => {
                match next {
                    LenientApprovalExceptionStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientApprovalExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient approval exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient approval exception", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.cost - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientApprovalException {
        LenientApprovalException {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            version: self.version,
            cost: self.cost,
            maximum: self.maximum,
            created_at: self.created_at,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient approval exception in status open.
pub fn new_lenient_approval_exception(code: &str, label: &str) -> LenientApprovalException {
    LenientApprovalException {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        version: 0,
        cost: 0.0,
        maximum: 0.0,
        created_at: 0,
        attempts: 0,
        status: LenientApprovalExceptionStatus::Open,
    }
}
