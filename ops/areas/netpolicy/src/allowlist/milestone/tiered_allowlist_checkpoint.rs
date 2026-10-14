use super::*;

use acme_kit::text;

/// Where a tiered allowlist checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAllowlistCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredAllowlistCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredAllowlistCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered allowlist checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistCheckpoint {
    pub code: String,
    pub name: String,
    pub title: String,
    pub tags: Vec<String>,
    pub locale: String,
    pub issuer: String,
    pub scope: String,
    pub capacity: i64,
    pub minimum: f64,
    pub active: bool,
    pub status: TieredAllowlistCheckpointStatus,
}

impl TieredAllowlistCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAllowlistCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAllowlistCheckpointStatus) -> bool {
        match self.status {
            TieredAllowlistCheckpointStatus::Created => {
                return next == TieredAllowlistCheckpointStatus::Validated
                    || next == TieredAllowlistCheckpointStatus::Retired;
            }
            TieredAllowlistCheckpointStatus::Validated => {
                match next {
                    TieredAllowlistCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAllowlistCheckpointStatus::Published => {
                return next == TieredAllowlistCheckpointStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredAllowlistCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered allowlist checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered allowlist checkpoint", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.capacity as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAllowlistCheckpoint {
        TieredAllowlistCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            locale: self.locale.clone(),
            issuer: self.issuer.clone(),
            scope: self.scope.clone(),
            capacity: self.capacity,
            minimum: self.minimum,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tiered allowlist checkpoint in status created.
pub fn new_tiered_allowlist_checkpoint(code: &str, name: &str) -> TieredAllowlistCheckpoint {
    TieredAllowlistCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        tags: Vec::new(),
        locale: "".to_string(),
        issuer: "".to_string(),
        scope: "".to_string(),
        capacity: 0,
        minimum: 0.0,
        active: false,
        status: TieredAllowlistCheckpointStatus::Created,
    }
}
