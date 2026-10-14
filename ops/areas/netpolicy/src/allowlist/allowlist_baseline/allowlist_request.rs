use super::*;

use acme_kit::text;

/// Where a allowlist request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistRequestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AllowlistRequestStatus {
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

impl std::fmt::Display for AllowlistRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRequest {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub ratio: f64,
    pub pinned: bool,
    pub amount: f64,
    pub priority: i64,
    pub capacity: i64,
    pub namespace: String,
    pub status: AllowlistRequestStatus,
}

impl AllowlistRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistRequestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistRequestStatus) -> bool {
        match self.status {
            AllowlistRequestStatus::Open => {
                return next == AllowlistRequestStatus::InReview || next == AllowlistRequestStatus::Reopened;
            }
            AllowlistRequestStatus::InReview => {
                match next {
                    AllowlistRequestStatus::Resolved | AllowlistRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistRequestStatus::Resolved => {
                return next == AllowlistRequestStatus::Archived;
            }
            AllowlistRequestStatus::Reopened => {
                return next == AllowlistRequestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.amount + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistRequest {
        AllowlistRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            ratio: self.ratio,
            pinned: self.pinned,
            amount: self.amount,
            priority: self.priority,
            capacity: self.capacity,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a allowlist request in status open.
pub fn new_allowlist_request(code: &str, label: &str) -> AllowlistRequest {
    AllowlistRequest {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        ratio: 0.0,
        pinned: false,
        amount: 0.0,
        priority: 0,
        capacity: 0,
        namespace: "".to_string(),
        status: AllowlistRequestStatus::Open,
    }
}
