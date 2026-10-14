use super::*;

use acme_kit::text;

/// Where a data export draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DataExportDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DataExportDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export draft as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportDraft {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub region: String,
    pub visible: bool,
    pub status: DataExportDraftStatus,
}

impl DataExportDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportDraftStatus) -> bool {
        match self.status {
            DataExportDraftStatus::Open => {
                return next == DataExportDraftStatus::Archived;
            }
            DataExportDraftStatus::InReview => {
                match next {
                    DataExportDraftStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportDraftStatus::Resolved => {
                return next == DataExportDraftStatus::Reopened || next == DataExportDraftStatus::Archived;
            }
            DataExportDraftStatus::Reopened => {
                return next == DataExportDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> DataExportDraft {
        DataExportDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            region: self.region.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a data export draft in status open.
pub fn new_data_export_draft(code: &str, name: &str) -> DataExportDraft {
    DataExportDraft {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        region: "".to_string(),
        visible: false,
        status: DataExportDraftStatus::Open,
    }
}
