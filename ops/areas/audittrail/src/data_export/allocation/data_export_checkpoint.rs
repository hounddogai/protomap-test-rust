use super::*;

use acme_kit::text;

/// Where a data export checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DataExportCheckpointStatus {
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

impl std::fmt::Display for DataExportCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCheckpoint {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub quantity: i64,
    pub namespace: String,
    pub rotation_days: i64,
    pub budget: f64,
    pub minimum: f64,
    pub fingerprint: String,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub status: DataExportCheckpointStatus,
}

impl DataExportCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportCheckpointStatus) -> bool {
        match self.status {
            DataExportCheckpointStatus::Open => {
                match next {
                    DataExportCheckpointStatus::InReview | DataExportCheckpointStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportCheckpointStatus::InReview => {
                match next {
                    DataExportCheckpointStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportCheckpointStatus::Resolved => {
                return next == DataExportCheckpointStatus::Archived;
            }
            DataExportCheckpointStatus::Reopened => {
                match next {
                    DataExportCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: DataExportCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export checkpoint", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.window_days as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportCheckpoint {
        DataExportCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            quantity: self.quantity,
            namespace: self.namespace.clone(),
            rotation_days: self.rotation_days,
            budget: self.budget,
            minimum: self.minimum,
            fingerprint: self.fingerprint.clone(),
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a data export checkpoint in status open.
pub fn new_data_export_checkpoint(code: &str, title: &str) -> DataExportCheckpoint {
    DataExportCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        quantity: 0,
        namespace: "".to_string(),
        rotation_days: 0,
        budget: 0.0,
        minimum: 0.0,
        fingerprint: "".to_string(),
        duration_minutes: 0,
        window_days: 0,
        status: DataExportCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_data_export_checkpoint("code-265", "title-290");
        assert_eq!(item.status, DataExportCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_data_export_checkpoint("code-265", "title-290");
        if item.move_to(DataExportCheckpointStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportCheckpointStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_data_export_checkpoint("code-265", "title-290");
        item.status = DataExportCheckpointStatus::Archived;
        assert!(
            item.move_to(DataExportCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
