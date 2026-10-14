use super::*;

use acme_kit::text;

/// Where a canary exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CanaryExceptionStatus {
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

impl std::fmt::Display for CanaryExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryException {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub rate: f64,
    pub owner_id: String,
    pub rotation_days: i64,
    pub channel: String,
    pub status: CanaryExceptionStatus,
}

impl CanaryException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryExceptionStatus) -> bool {
        match self.status {
            CanaryExceptionStatus::Open => {
                match next {
                    CanaryExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryExceptionStatus::InReview => {
                return next == CanaryExceptionStatus::Reopened || next == CanaryExceptionStatus::Archived;
            }
            CanaryExceptionStatus::Resolved => {
                return next == CanaryExceptionStatus::Archived;
            }
            CanaryExceptionStatus::Reopened => {
                return next == CanaryExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.capacity as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryException {
        CanaryException {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            rate: self.rate,
            owner_id: self.owner_id.clone(),
            rotation_days: self.rotation_days,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a canary exception in status open.
pub fn new_canary_exception(code: &str, label: &str) -> CanaryException {
    CanaryException {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        rate: 0.0,
        owner_id: "".to_string(),
        rotation_days: 0,
        channel: "".to_string(),
        status: CanaryExceptionStatus::Open,
    }
}
