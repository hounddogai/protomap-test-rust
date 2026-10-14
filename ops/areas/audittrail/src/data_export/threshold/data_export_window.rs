use super::*;

use acme_kit::text;

/// Where a data export window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DataExportWindowStatus {
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

impl std::fmt::Display for DataExportWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export window as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportWindow {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub cost: f64,
    pub contact_email: String,
    pub capacity: i64,
    pub channel: String,
    pub revision: i64,
    pub status: DataExportWindowStatus,
}

impl DataExportWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportWindowStatus) -> bool {
        match self.status {
            DataExportWindowStatus::New => {
                match next {
                    DataExportWindowStatus::Running | DataExportWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportWindowStatus::Queued => {
                return next == DataExportWindowStatus::Failed;
            }
            DataExportWindowStatus::Running => {
                return next == DataExportWindowStatus::Failed;
            }
            DataExportWindowStatus::Done => {
                match next {
                    DataExportWindowStatus::Failed => {
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

    pub fn move_to(&mut self, next: DataExportWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export window", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.capacity as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportWindow {
        DataExportWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            cost: self.cost,
            contact_email: self.contact_email.clone(),
            capacity: self.capacity,
            channel: self.channel.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a data export window in status new.
pub fn new_data_export_window(code: &str, name: &str) -> DataExportWindow {
    DataExportWindow {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        cost: 0.0,
        contact_email: "".to_string(),
        capacity: 0,
        channel: "".to_string(),
        revision: 0,
        status: DataExportWindowStatus::New,
    }
}
