use super::*;

use acme_kit::text;

/// Where a migration draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDraft {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub description: String,
    pub category: String,
    pub quantity: i64,
    pub status: MigrationDraftStatus,
}

impl MigrationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationDraftStatus) -> bool {
        match self.status {
            MigrationDraftStatus::Created => {
                return next == MigrationDraftStatus::Validated || next == MigrationDraftStatus::Published;
            }
            MigrationDraftStatus::Validated => {
                return next == MigrationDraftStatus::Published;
            }
            MigrationDraftStatus::Published => {
                return next == MigrationDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> MigrationDraft {
        MigrationDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a migration draft in status created.
pub fn new_migration_draft(code: &str, label: &str) -> MigrationDraft {
    MigrationDraft {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        description: "".to_string(),
        category: "".to_string(),
        quantity: 0,
        status: MigrationDraftStatus::Created,
    }
}
