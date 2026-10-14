use super::*;

use acme_kit::text;

/// Where a batched identity rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedIdentityRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedIdentityRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedIdentityRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched identity rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollup {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub starts_at: i64,
    pub threshold: f64,
    pub priority: i64,
    pub cipher: String,
    pub status: BatchedIdentityRollupStatus,
}

impl BatchedIdentityRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedIdentityRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedIdentityRollupStatus) -> bool {
        match self.status {
            BatchedIdentityRollupStatus::Requested => {
                match next {
                    BatchedIdentityRollupStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedIdentityRollupStatus::Granted => {
                return next == BatchedIdentityRollupStatus::Expired;
            }
            BatchedIdentityRollupStatus::Revoked => {
                return next == BatchedIdentityRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedIdentityRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched identity rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched identity rollup", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.starts_at as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedIdentityRollup {
        BatchedIdentityRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            starts_at: self.starts_at,
            threshold: self.threshold,
            priority: self.priority,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched identity rollup in status requested.
pub fn new_batched_identity_rollup(code: &str, title: &str) -> BatchedIdentityRollup {
    BatchedIdentityRollup {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        starts_at: 0,
        threshold: 0.0,
        priority: 0,
        cipher: "".to_string(),
        status: BatchedIdentityRollupStatus::Requested,
    }
}
