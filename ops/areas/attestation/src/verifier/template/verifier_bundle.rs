use super::*;

use acme_kit::text;

/// Where a verifier bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl VerifierBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for VerifierBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundle {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub key_id: String,
    pub minimum: f64,
    pub created_at: i64,
    pub expires_at: i64,
    pub sequence: i64,
    pub ttl_seconds: i64,
    pub source: String,
    pub status: VerifierBundleStatus,
}

impl VerifierBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierBundleStatus) -> bool {
        match self.status {
            VerifierBundleStatus::Created => {
                return next == VerifierBundleStatus::Published || next == VerifierBundleStatus::Retired;
            }
            VerifierBundleStatus::Validated => {
                match next {
                    VerifierBundleStatus::Published | VerifierBundleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierBundleStatus::Published => {
                return next == VerifierBundleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "verifier bundle", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.sequence as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierBundle {
        VerifierBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            created_at: self.created_at,
            expires_at: self.expires_at,
            sequence: self.sequence,
            ttl_seconds: self.ttl_seconds,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a verifier bundle in status created.
pub fn new_verifier_bundle(code: &str, label: &str) -> VerifierBundle {
    VerifierBundle {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        key_id: "".to_string(),
        minimum: 0.0,
        created_at: 0,
        expires_at: 0,
        sequence: 0,
        ttl_seconds: 0,
        source: "".to_string(),
        status: VerifierBundleStatus::Created,
    }
}
