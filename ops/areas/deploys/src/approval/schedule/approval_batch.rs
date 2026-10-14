use super::*;

use acme_kit::text;

/// Where a approval batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ApprovalBatchStatus {
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

impl std::fmt::Display for ApprovalBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBatch {
    pub code: String,
    pub name: String,
    pub description: String,
    pub discount: f64,
    pub phone: String,
    pub limit: i64,
    pub maximum: f64,
    pub category: String,
    pub fingerprint: String,
    pub status: ApprovalBatchStatus,
}

impl ApprovalBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalBatchStatus) -> bool {
        match self.status {
            ApprovalBatchStatus::Planned => {
                match next {
                    ApprovalBatchStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalBatchStatus::Scheduled => {
                match next {
                    ApprovalBatchStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalBatchStatus::InProgress => {
                match next {
                    ApprovalBatchStatus::Completed | ApprovalBatchStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalBatchStatus::Completed => {
                return next == ApprovalBatchStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "approval batch", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.limit as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalBatch {
        ApprovalBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            discount: self.discount,
            phone: self.phone.clone(),
            limit: self.limit,
            maximum: self.maximum,
            category: self.category.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a approval batch in status planned.
pub fn new_approval_batch(code: &str, name: &str) -> ApprovalBatch {
    ApprovalBatch {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        discount: 0.0,
        phone: "".to_string(),
        limit: 0,
        maximum: 0.0,
        category: "".to_string(),
        fingerprint: "".to_string(),
        status: ApprovalBatchStatus::Planned,
    }
}
