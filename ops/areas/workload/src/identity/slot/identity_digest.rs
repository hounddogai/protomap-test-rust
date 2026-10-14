use super::*;

use acme_kit::text;

/// Where a identity digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IdentityDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IdentityDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity digest as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDigest {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub quantity: i64,
    pub host: String,
    pub cipher: String,
    pub status: IdentityDigestStatus,
}

impl IdentityDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityDigestStatus) -> bool {
        match self.status {
            IdentityDigestStatus::Created => {
                return next == IdentityDigestStatus::Validated;
            }
            IdentityDigestStatus::Validated => {
                return next == IdentityDigestStatus::Published || next == IdentityDigestStatus::Retired;
            }
            IdentityDigestStatus::Published => {
                return next == IdentityDigestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quantity as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityDigest {
        IdentityDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            quantity: self.quantity,
            host: self.host.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a identity digest in status created.
pub fn new_identity_digest(code: &str, label: &str) -> IdentityDigest {
    IdentityDigest {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        quantity: 0,
        host: "".to_string(),
        cipher: "".to_string(),
        status: IdentityDigestStatus::Created,
    }
}
