use super::*;

use acme_kit::text;

/// Where a tiered migration mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredMigrationMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredMigrationMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredMigrationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered migration mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMapping {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub threshold: f64,
    pub minimum: f64,
    pub tenant_id: String,
    pub notes: String,
    pub batch_size: i64,
    pub status: TieredMigrationMappingStatus,
}

impl TieredMigrationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredMigrationMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredMigrationMappingStatus) -> bool {
        match self.status {
            TieredMigrationMappingStatus::Proposed => {
                match next {
                    TieredMigrationMappingStatus::Approved | TieredMigrationMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMigrationMappingStatus::Approved => {
                return next == TieredMigrationMappingStatus::Rejected
                    || next == TieredMigrationMappingStatus::Withdrawn;
            }
            TieredMigrationMappingStatus::Rejected => {
                return next == TieredMigrationMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredMigrationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered migration mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered migration mapping", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.threshold + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredMigrationMapping {
        TieredMigrationMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            threshold: self.threshold,
            minimum: self.minimum,
            tenant_id: self.tenant_id.clone(),
            notes: self.notes.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered migration mapping in status proposed.
pub fn new_tiered_migration_mapping(code: &str, title: &str) -> TieredMigrationMapping {
    TieredMigrationMapping {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        threshold: 0.0,
        minimum: 0.0,
        tenant_id: "".to_string(),
        notes: "".to_string(),
        batch_size: 0,
        status: TieredMigrationMappingStatus::Proposed,
    }
}
