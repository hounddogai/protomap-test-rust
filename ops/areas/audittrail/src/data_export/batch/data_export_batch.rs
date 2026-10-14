use super::*;

use acme_kit::text;

/// Where a data export batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DataExportBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DataExportBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export batch as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportBatch {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub owner_id: String,
    pub created_at: i64,
    pub duration_minutes: i64,
    pub audience: String,
    pub rate: f64,
    pub version: i64,
    pub status: DataExportBatchStatus,
}

impl DataExportBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportBatchStatus) -> bool {
        match self.status {
            DataExportBatchStatus::Idle => {
                match next {
                    DataExportBatchStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportBatchStatus::Warming => {
                return next == DataExportBatchStatus::Stopped;
            }
            DataExportBatchStatus::Ready => {
                match next {
                    DataExportBatchStatus::Draining | DataExportBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportBatchStatus::Draining => {
                match next {
                    DataExportBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DataExportBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportBatch {
        DataExportBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            owner_id: self.owner_id.clone(),
            created_at: self.created_at,
            duration_minutes: self.duration_minutes,
            audience: self.audience.clone(),
            rate: self.rate,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a data export batch in status idle.
pub fn new_data_export_batch(code: &str, label: &str) -> DataExportBatch {
    DataExportBatch {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        owner_id: "".to_string(),
        created_at: 0,
        duration_minutes: 0,
        audience: "".to_string(),
        rate: 0.0,
        version: 0,
        status: DataExportBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_data_export_batch("code-260", "label-509");
        assert_eq!(item.status, DataExportBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_data_export_batch("code-260", "label-509");
        if item.move_to(DataExportBatchStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportBatchStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_data_export_batch("code-260", "label-509");
        item.status = DataExportBatchStatus::Stopped;
        assert!(
            item.move_to(DataExportBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
