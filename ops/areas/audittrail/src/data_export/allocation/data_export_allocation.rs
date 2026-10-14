use super::*;

use acme_kit::text;

/// Where a data export allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DataExportAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for DataExportAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAllocation {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub notes: String,
    pub checksum: String,
    pub window_days: i64,
    pub duration_minutes: i64,
    pub audience: String,
    pub visible: bool,
    pub namespace: String,
    pub ends_at: i64,
    pub status: DataExportAllocationStatus,
}

impl DataExportAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportAllocationStatus) -> bool {
        match self.status {
            DataExportAllocationStatus::New => {
                match next {
                    DataExportAllocationStatus::Queued | DataExportAllocationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportAllocationStatus::Queued => {
                return next == DataExportAllocationStatus::Done || next == DataExportAllocationStatus::Failed;
            }
            DataExportAllocationStatus::Running => {
                return next == DataExportAllocationStatus::Done || next == DataExportAllocationStatus::Failed;
            }
            DataExportAllocationStatus::Done => {
                match next {
                    DataExportAllocationStatus::Failed => {
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

    pub fn move_to(&mut self, next: DataExportAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export allocation", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.window_days as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportAllocation {
        DataExportAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            notes: self.notes.clone(),
            checksum: self.checksum.clone(),
            window_days: self.window_days,
            duration_minutes: self.duration_minutes,
            audience: self.audience.clone(),
            visible: self.visible,
            namespace: self.namespace.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a data export allocation in status new.
pub fn new_data_export_allocation(code: &str, title: &str) -> DataExportAllocation {
    DataExportAllocation {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        notes: "".to_string(),
        checksum: "".to_string(),
        window_days: 0,
        duration_minutes: 0,
        audience: "".to_string(),
        visible: false,
        namespace: "".to_string(),
        ends_at: 0,
        status: DataExportAllocationStatus::New,
    }
}
