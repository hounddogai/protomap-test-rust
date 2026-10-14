use super::*;

use acme_kit::text;

/// Where a data export threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportThresholdStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DataExportThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DataExportThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThreshold {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub cost: f64,
    pub checksum: String,
    pub created_at: i64,
    pub fingerprint: String,
    pub reviewer: String,
    pub status: DataExportThresholdStatus,
}

impl DataExportThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportThresholdStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportThresholdStatus) -> bool {
        match self.status {
            DataExportThresholdStatus::Planned => {
                return next == DataExportThresholdStatus::Scheduled || next == DataExportThresholdStatus::InProgress;
            }
            DataExportThresholdStatus::Scheduled => {
                match next {
                    DataExportThresholdStatus::InProgress | DataExportThresholdStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportThresholdStatus::InProgress => {
                return next == DataExportThresholdStatus::Cancelled;
            }
            DataExportThresholdStatus::Completed => {
                match next {
                    DataExportThresholdStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DataExportThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export threshold {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        (self.cost * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportThreshold {
        DataExportThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            cost: self.cost,
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a data export threshold in status planned.
pub fn new_data_export_threshold(code: &str, label: &str) -> DataExportThreshold {
    DataExportThreshold {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        cost: 0.0,
        checksum: "".to_string(),
        created_at: 0,
        fingerprint: "".to_string(),
        reviewer: "".to_string(),
        status: DataExportThresholdStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_data_export_threshold("code-316", "label-543");
        assert_eq!(item.status, DataExportThresholdStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_data_export_threshold("code-316", "label-543");
        if item.move_to(DataExportThresholdStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportThresholdStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_data_export_threshold("code-316", "label-543");
        item.status = DataExportThresholdStatus::Cancelled;
        assert!(
            item.move_to(DataExportThresholdStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
