use super::*;

use acme_kit::text;

/// Where a manual data export window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDataExportWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualDataExportWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualDataExportWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual data export window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportWindow {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub quota_units: i64,
    pub archived: bool,
    pub status: ManualDataExportWindowStatus,
}

impl ManualDataExportWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDataExportWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDataExportWindowStatus) -> bool {
        match self.status {
            ManualDataExportWindowStatus::Created => {
                match next {
                    ManualDataExportWindowStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportWindowStatus::Validated => {
                return next == ManualDataExportWindowStatus::Published
                    || next == ManualDataExportWindowStatus::Retired;
            }
            ManualDataExportWindowStatus::Published => {
                match next {
                    ManualDataExportWindowStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualDataExportWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual data export window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quota_units as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDataExportWindow {
        ManualDataExportWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            quota_units: self.quota_units,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual data export window in status created.
pub fn new_manual_data_export_window(code: &str, title: &str) -> ManualDataExportWindow {
    ManualDataExportWindow {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        ttl_seconds: 0,
        quota_units: 0,
        archived: false,
        status: ManualDataExportWindowStatus::Created,
    }
}
