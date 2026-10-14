use super::*;

use acme_kit::text;

/// Where a migration threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MigrationThresholdStatus {
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

impl std::fmt::Display for MigrationThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration threshold as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationThreshold {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub scope: String,
    pub namespace: String,
    pub status: MigrationThresholdStatus,
}

impl MigrationThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationThresholdStatus) -> bool {
        match self.status {
            MigrationThresholdStatus::Draft => {
                return next == MigrationThresholdStatus::Pending;
            }
            MigrationThresholdStatus::Pending => {
                match next {
                    MigrationThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationThresholdStatus::Active => {
                return next == MigrationThresholdStatus::Closed;
            }
            MigrationThresholdStatus::Suspended => {
                return next == MigrationThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "migration threshold", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> MigrationThreshold {
        MigrationThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            scope: self.scope.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a migration threshold in status draft.
pub fn new_migration_threshold(code: &str, label: &str) -> MigrationThreshold {
    MigrationThreshold {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        scope: "".to_string(),
        namespace: "".to_string(),
        status: MigrationThresholdStatus::Draft,
    }
}
