use super::*;

use acme_kit::text;

/// Where a nonce target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl NonceTargetStatus {
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

impl std::fmt::Display for NonceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceTarget {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub batch_size: i64,
    pub issuer: String,
    pub maximum: f64,
    pub tenant_id: String,
    pub summary: String,
    pub country: String,
    pub external_ref: String,
    pub status: NonceTargetStatus,
}

impl NonceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceTargetStatus) -> bool {
        match self.status {
            NonceTargetStatus::Open => {
                match next {
                    NonceTargetStatus::Resolved | NonceTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceTargetStatus::InReview => {
                return next == NonceTargetStatus::Resolved;
            }
            NonceTargetStatus::Resolved => {
                return next == NonceTargetStatus::Reopened;
            }
            NonceTargetStatus::Reopened => {
                return next == NonceTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce target", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.maximum * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceTarget {
        NonceTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            tenant_id: self.tenant_id.clone(),
            summary: self.summary.clone(),
            country: self.country.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a nonce target in status open.
pub fn new_nonce_target(code: &str, name: &str) -> NonceTarget {
    NonceTarget {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        batch_size: 0,
        issuer: "".to_string(),
        maximum: 0.0,
        tenant_id: "".to_string(),
        summary: "".to_string(),
        country: "".to_string(),
        external_ref: "".to_string(),
        status: NonceTargetStatus::Open,
    }
}
