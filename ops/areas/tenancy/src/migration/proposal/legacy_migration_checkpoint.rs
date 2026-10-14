use super::*;

use acme_kit::text;

/// Where a legacy migration checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyMigrationCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyMigrationCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyMigrationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy migration checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationCheckpoint {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub window_days: i64,
    pub scope: String,
    pub timezone: String,
    pub ceiling: f64,
    pub status: LegacyMigrationCheckpointStatus,
}

impl LegacyMigrationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyMigrationCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyMigrationCheckpointStatus) -> bool {
        match self.status {
            LegacyMigrationCheckpointStatus::Requested => {
                return next == LegacyMigrationCheckpointStatus::Expired;
            }
            LegacyMigrationCheckpointStatus::Granted => {
                match next {
                    LegacyMigrationCheckpointStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMigrationCheckpointStatus::Revoked => {
                return next == LegacyMigrationCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyMigrationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy migration checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.window_days as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyMigrationCheckpoint {
        LegacyMigrationCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            window_days: self.window_days,
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy migration checkpoint in status requested.
pub fn new_legacy_migration_checkpoint(code: &str, label: &str) -> LegacyMigrationCheckpoint {
    LegacyMigrationCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        window_days: 0,
        scope: "".to_string(),
        timezone: "".to_string(),
        ceiling: 0.0,
        status: LegacyMigrationCheckpointStatus::Requested,
    }
}
