use super::*;

use acme_kit::text;

/// Where a approval request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalRequestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ApprovalRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ApprovalRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval request as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRequest {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub capacity: i64,
    pub issuer: String,
    pub status: ApprovalRequestStatus,
}

impl ApprovalRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalRequestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalRequestStatus) -> bool {
        match self.status {
            ApprovalRequestStatus::Planned => {
                match next {
                    ApprovalRequestStatus::InProgress | ApprovalRequestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalRequestStatus::Scheduled => {
                return next == ApprovalRequestStatus::InProgress || next == ApprovalRequestStatus::Completed;
            }
            ApprovalRequestStatus::InProgress => {
                return next == ApprovalRequestStatus::Completed;
            }
            ApprovalRequestStatus::Completed => {
                match next {
                    ApprovalRequestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ApprovalRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> ApprovalRequest {
        ApprovalRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            capacity: self.capacity,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a approval request in status planned.
pub fn new_approval_request(code: &str, title: &str) -> ApprovalRequest {
    ApprovalRequest {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        capacity: 0,
        issuer: "".to_string(),
        status: ApprovalRequestStatus::Planned,
    }
}
