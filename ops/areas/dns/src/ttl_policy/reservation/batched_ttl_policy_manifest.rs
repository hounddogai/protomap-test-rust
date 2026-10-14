use super::*;

use acme_kit::text;

/// Where a batched ttl policy manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTtlPolicyManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedTtlPolicyManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedTtlPolicyManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched ttl policy manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyManifest {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub cost: f64,
    pub created_at: i64,
    pub key_id: String,
    pub locale: String,
    pub cluster: String,
    pub score: f64,
    pub status: BatchedTtlPolicyManifestStatus,
}

impl BatchedTtlPolicyManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTtlPolicyManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTtlPolicyManifestStatus) -> bool {
        match self.status {
            BatchedTtlPolicyManifestStatus::Requested => {
                match next {
                    BatchedTtlPolicyManifestStatus::Granted | BatchedTtlPolicyManifestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTtlPolicyManifestStatus::Granted => {
                return next == BatchedTtlPolicyManifestStatus::Revoked;
            }
            BatchedTtlPolicyManifestStatus::Revoked => {
                return next == BatchedTtlPolicyManifestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTtlPolicyManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched ttl policy manifest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.cost - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTtlPolicyManifest {
        BatchedTtlPolicyManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            cost: self.cost,
            created_at: self.created_at,
            key_id: self.key_id.clone(),
            locale: self.locale.clone(),
            cluster: self.cluster.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched ttl policy manifest in status requested.
pub fn new_batched_ttl_policy_manifest(code: &str, name: &str) -> BatchedTtlPolicyManifest {
    BatchedTtlPolicyManifest {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        cost: 0.0,
        created_at: 0,
        key_id: "".to_string(),
        locale: "".to_string(),
        cluster: "".to_string(),
        score: 0.0,
        status: BatchedTtlPolicyManifestStatus::Requested,
    }
}
