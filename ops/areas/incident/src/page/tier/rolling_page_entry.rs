use super::*;

use acme_kit::text;

/// Where a rolling page entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPageEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingPageEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingPageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling page entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageEntry {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub active: bool,
    pub audience: String,
    pub category: String,
    pub quantity: i64,
    pub status: RollingPageEntryStatus,
}

impl RollingPageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPageEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPageEntryStatus) -> bool {
        match self.status {
            RollingPageEntryStatus::Created => {
                match next {
                    RollingPageEntryStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPageEntryStatus::Validated => {
                match next {
                    RollingPageEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPageEntryStatus::Published => {
                return next == RollingPageEntryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling page entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling page entry", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> RollingPageEntry {
        RollingPageEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            active: self.active,
            audience: self.audience.clone(),
            category: self.category.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling page entry in status created.
pub fn new_rolling_page_entry(code: &str, title: &str) -> RollingPageEntry {
    RollingPageEntry {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        active: false,
        audience: "".to_string(),
        category: "".to_string(),
        quantity: 0,
        status: RollingPageEntryStatus::Created,
    }
}
