use super::*;

use acme_kit::text;

/// Where a approval digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ApprovalDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ApprovalDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval digest as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDigest {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub attempts: i64,
    pub rate: f64,
    pub grace_days: i64,
    pub status: ApprovalDigestStatus,
}

impl ApprovalDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalDigestStatus) -> bool {
        match self.status {
            ApprovalDigestStatus::Idle => {
                match next {
                    ApprovalDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalDigestStatus::Warming => {
                match next {
                    ApprovalDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalDigestStatus::Ready => {
                return next == ApprovalDigestStatus::Stopped;
            }
            ApprovalDigestStatus::Draining => {
                return next == ApprovalDigestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalDigest {
        ApprovalDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            attempts: self.attempts,
            rate: self.rate,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a approval digest in status idle.
pub fn new_approval_digest(code: &str, title: &str) -> ApprovalDigest {
    ApprovalDigest {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        attempts: 0,
        rate: 0.0,
        grace_days: 0,
        status: ApprovalDigestStatus::Idle,
    }
}
