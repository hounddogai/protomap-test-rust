use super::*;

use acme_kit::text;

/// Where a data export record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DataExportRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DataExportRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRecord {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub grace_days: i64,
    pub tags: Vec<String>,
    pub quota_units: i64,
    pub category: String,
    pub status: DataExportRecordStatus,
}

impl DataExportRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportRecordStatus) -> bool {
        match self.status {
            DataExportRecordStatus::Created => {
                return next == DataExportRecordStatus::Validated || next == DataExportRecordStatus::Retired;
            }
            DataExportRecordStatus::Validated => {
                match next {
                    DataExportRecordStatus::Published | DataExportRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportRecordStatus::Published => {
                match next {
                    DataExportRecordStatus::Retired => {
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

    pub fn move_to(&mut self, next: DataExportRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export record", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.grace_days as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportRecord {
        DataExportRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            grace_days: self.grace_days,
            tags: self.tags.clone(),
            quota_units: self.quota_units,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a data export record in status created.
pub fn new_data_export_record(code: &str, name: &str) -> DataExportRecord {
    DataExportRecord {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        grace_days: 0,
        tags: Vec::new(),
        quota_units: 0,
        category: "".to_string(),
        status: DataExportRecordStatus::Created,
    }
}
