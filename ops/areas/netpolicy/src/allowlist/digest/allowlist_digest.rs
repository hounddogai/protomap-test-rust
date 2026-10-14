use super::*;

use acme_kit::text;

/// Where a allowlist digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AllowlistDigestStatus {
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

impl std::fmt::Display for AllowlistDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigest {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub principal: String,
    pub audience: String,
    pub category: String,
    pub quantity: i64,
    pub cipher: String,
    pub window_days: i64,
    pub status: AllowlistDigestStatus,
}

impl AllowlistDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistDigestStatus) -> bool {
        match self.status {
            AllowlistDigestStatus::Open => {
                return next == AllowlistDigestStatus::Archived;
            }
            AllowlistDigestStatus::InReview => {
                match next {
                    AllowlistDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistDigestStatus::Resolved => {
                return next == AllowlistDigestStatus::Reopened;
            }
            AllowlistDigestStatus::Reopened => {
                match next {
                    AllowlistDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: AllowlistDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.burst as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistDigest {
        AllowlistDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            principal: self.principal.clone(),
            audience: self.audience.clone(),
            category: self.category.clone(),
            quantity: self.quantity,
            cipher: self.cipher.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a allowlist digest in status open.
pub fn new_allowlist_digest(code: &str, title: &str) -> AllowlistDigest {
    AllowlistDigest {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        principal: "".to_string(),
        audience: "".to_string(),
        category: "".to_string(),
        quantity: 0,
        cipher: "".to_string(),
        window_days: 0,
        status: AllowlistDigestStatus::Open,
    }
}
