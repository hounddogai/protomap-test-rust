use super::*;

use acme_kit::text;

/// Where a federation manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationManifestStatus {
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

impl std::fmt::Display for FederationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationManifest {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub priority: i64,
    pub created_at: i64,
    pub audience: String,
    pub cipher: String,
    pub fingerprint: String,
    pub status: FederationManifestStatus,
}

impl FederationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationManifestStatus) -> bool {
        match self.status {
            FederationManifestStatus::Open => {
                return next == FederationManifestStatus::Reopened;
            }
            FederationManifestStatus::InReview => {
                return next == FederationManifestStatus::Archived;
            }
            FederationManifestStatus::Resolved => {
                match next {
                    FederationManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationManifestStatus::Reopened => {
                return next == FederationManifestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.created_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationManifest {
        FederationManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            priority: self.priority,
            created_at: self.created_at,
            audience: self.audience.clone(),
            cipher: self.cipher.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation manifest in status open.
pub fn new_federation_manifest(code: &str, title: &str) -> FederationManifest {
    FederationManifest {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        priority: 0,
        created_at: 0,
        audience: "".to_string(),
        cipher: "".to_string(),
        fingerprint: "".to_string(),
        status: FederationManifestStatus::Open,
    }
}
