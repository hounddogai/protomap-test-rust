use super::*;

use acme_kit::text;

/// Where a grant bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantBundleStatus {
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

impl std::fmt::Display for GrantBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantBundle {
    pub code: String,
    pub name: String,
    pub description: String,
    pub ttl_seconds: i64,
    pub cipher: String,
    pub issuer: String,
    pub owner_id: String,
    pub status: GrantBundleStatus,
}

impl GrantBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantBundleStatus) -> bool {
        match self.status {
            GrantBundleStatus::Open => {
                match next {
                    GrantBundleStatus::InReview | GrantBundleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantBundleStatus::InReview => {
                return next == GrantBundleStatus::Reopened;
            }
            GrantBundleStatus::Resolved => {
                match next {
                    GrantBundleStatus::Reopened | GrantBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantBundleStatus::Reopened => {
                return next == GrantBundleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> GrantBundle {
        GrantBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            ttl_seconds: self.ttl_seconds,
            cipher: self.cipher.clone(),
            issuer: self.issuer.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a grant bundle in status open.
pub fn new_grant_bundle(code: &str, name: &str) -> GrantBundle {
    GrantBundle {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        ttl_seconds: 0,
        cipher: "".to_string(),
        issuer: "".to_string(),
        owner_id: "".to_string(),
        status: GrantBundleStatus::Open,
    }
}
