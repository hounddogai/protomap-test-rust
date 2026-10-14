use super::*;

use acme_kit::text;

/// Where a migration entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MigrationEntryStatus {
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

impl std::fmt::Display for MigrationEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationEntry {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub region: String,
    pub checksum: String,
    pub reviewer: String,
    pub description: String,
    pub floor_value: f64,
    pub status: MigrationEntryStatus,
}

impl MigrationEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationEntryStatus) -> bool {
        match self.status {
            MigrationEntryStatus::Draft => {
                return next == MigrationEntryStatus::Closed;
            }
            MigrationEntryStatus::Pending => {
                return next == MigrationEntryStatus::Closed;
            }
            MigrationEntryStatus::Active => {
                return next == MigrationEntryStatus::Suspended;
            }
            MigrationEntryStatus::Suspended => {
                return next == MigrationEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "migration entry", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.revision as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationEntry {
        MigrationEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            region: self.region.clone(),
            checksum: self.checksum.clone(),
            reviewer: self.reviewer.clone(),
            description: self.description.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a migration entry in status draft.
pub fn new_migration_entry(code: &str, label: &str) -> MigrationEntry {
    MigrationEntry {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        region: "".to_string(),
        checksum: "".to_string(),
        reviewer: "".to_string(),
        description: "".to_string(),
        floor_value: 0.0,
        status: MigrationEntryStatus::Draft,
    }
}
