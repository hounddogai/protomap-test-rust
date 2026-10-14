use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual nonce checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualNonceCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualNonceCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ManualNonceCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual nonce checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceCheckpoint {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub updated_at: i64,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub ratio: f64,
    pub status: ManualNonceCheckpointStatus,
}

impl ManualNonceCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualNonceCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualNonceCheckpointStatus) -> bool {
        match self.status {
            ManualNonceCheckpointStatus::Draft => {
                match next {
                    ManualNonceCheckpointStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualNonceCheckpointStatus::Pending => {
                match next {
                    ManualNonceCheckpointStatus::Active | ManualNonceCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualNonceCheckpointStatus::Active => {
                match next {
                    ManualNonceCheckpointStatus::Suspended | ManualNonceCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualNonceCheckpointStatus::Suspended => {
                match next {
                    ManualNonceCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualNonceCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual nonce checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ratio + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualNonceCheckpoint {
        ManualNonceCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            updated_at: self.updated_at,
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual nonce checkpoint in status draft.
pub fn new_manual_nonce_checkpoint(code: &str, name: &str) -> ManualNonceCheckpoint {
    ManualNonceCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        updated_at: 0,
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        ratio: 0.0,
        status: ManualNonceCheckpointStatus::Draft,
    }
}
