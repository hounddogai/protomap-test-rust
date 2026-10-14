use super::*;

use acme_kit::text;

/// Where a binding record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BindingRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BindingRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingRecord {
    pub code: String,
    pub name: String,
    pub country: String,
    pub retries: i64,
    pub cost: f64,
    pub checksum: String,
    pub status: BindingRecordStatus,
}

impl BindingRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingRecordStatus) -> bool {
        match self.status {
            BindingRecordStatus::Created => {
                return next == BindingRecordStatus::Validated;
            }
            BindingRecordStatus::Validated => {
                return next == BindingRecordStatus::Retired;
            }
            BindingRecordStatus::Published => {
                return next == BindingRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.cost + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingRecord {
        BindingRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            retries: self.retries,
            cost: self.cost,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a binding record in status created.
pub fn new_binding_record(code: &str, name: &str) -> BindingRecord {
    BindingRecord {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        retries: 0,
        cost: 0.0,
        checksum: "".to_string(),
        status: BindingRecordStatus::Created,
    }
}
