use super::*;

use acme_kit::text;

/// Where a tiered egress override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredEgressOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredEgressOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredEgressOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered egress override as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressOverride {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub threshold: f64,
    pub revision: i64,
    pub attempts: i64,
    pub cipher: String,
    pub archived: bool,
    pub status: TieredEgressOverrideStatus,
}

impl TieredEgressOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredEgressOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredEgressOverrideStatus) -> bool {
        match self.status {
            TieredEgressOverrideStatus::Created => {
                return next == TieredEgressOverrideStatus::Published || next == TieredEgressOverrideStatus::Retired;
            }
            TieredEgressOverrideStatus::Validated => {
                match next {
                    TieredEgressOverrideStatus::Published | TieredEgressOverrideStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredEgressOverrideStatus::Published => {
                return next == TieredEgressOverrideStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredEgressOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered egress override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredEgressOverride {
        TieredEgressOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            threshold: self.threshold,
            revision: self.revision,
            attempts: self.attempts,
            cipher: self.cipher.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered egress override in status created.
pub fn new_tiered_egress_override(code: &str, title: &str) -> TieredEgressOverride {
    TieredEgressOverride {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        threshold: 0.0,
        revision: 0,
        attempts: 0,
        cipher: "".to_string(),
        archived: false,
        status: TieredEgressOverrideStatus::Created,
    }
}
