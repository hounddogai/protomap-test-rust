use super::*;

use acme_kit::text;

/// Where a legacy allowlist checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAllowlistCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyAllowlistCheckpointStatus {
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

impl std::fmt::Display for LegacyAllowlistCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy allowlist checkpoint as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistCheckpoint {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub cipher: String,
    pub active: bool,
    pub name: String,
    pub expires_at: i64,
    pub archived: bool,
    pub status: LegacyAllowlistCheckpointStatus,
}

impl LegacyAllowlistCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAllowlistCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAllowlistCheckpointStatus) -> bool {
        match self.status {
            LegacyAllowlistCheckpointStatus::Draft => {
                match next {
                    LegacyAllowlistCheckpointStatus::Pending | LegacyAllowlistCheckpointStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistCheckpointStatus::Pending => {
                match next {
                    LegacyAllowlistCheckpointStatus::Active | LegacyAllowlistCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistCheckpointStatus::Active => {
                match next {
                    LegacyAllowlistCheckpointStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistCheckpointStatus::Suspended => {
                match next {
                    LegacyAllowlistCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyAllowlistCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy allowlist checkpoint {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.created_at as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAllowlistCheckpoint {
        LegacyAllowlistCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            cipher: self.cipher.clone(),
            active: self.active,
            name: self.name.clone(),
            expires_at: self.expires_at,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy allowlist checkpoint in status draft.
pub fn new_legacy_allowlist_checkpoint(code: &str, title: &str) -> LegacyAllowlistCheckpoint {
    LegacyAllowlistCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        cipher: "".to_string(),
        active: false,
        name: "".to_string(),
        expires_at: 0,
        archived: false,
        status: LegacyAllowlistCheckpointStatus::Draft,
    }
}
