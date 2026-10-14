use super::*;

use acme_kit::text;

/// Where a data export override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DataExportOverrideStatus {
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

impl std::fmt::Display for DataExportOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportOverride {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub approved: bool,
    pub updated_at: i64,
    pub quantity: i64,
    pub ttl_seconds: i64,
    pub status: DataExportOverrideStatus,
}

impl DataExportOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportOverrideStatus) -> bool {
        match self.status {
            DataExportOverrideStatus::Planned => {
                return next == DataExportOverrideStatus::Scheduled || next == DataExportOverrideStatus::Cancelled;
            }
            DataExportOverrideStatus::Scheduled => {
                return next == DataExportOverrideStatus::InProgress || next == DataExportOverrideStatus::Completed;
            }
            DataExportOverrideStatus::InProgress => {
                match next {
                    DataExportOverrideStatus::Completed | DataExportOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportOverrideStatus::Completed => {
                match next {
                    DataExportOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DataExportOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.updated_at as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportOverride {
        DataExportOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            approved: self.approved,
            updated_at: self.updated_at,
            quantity: self.quantity,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a data export override in status planned.
pub fn new_data_export_override(code: &str, name: &str) -> DataExportOverride {
    DataExportOverride {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        approved: false,
        updated_at: 0,
        quantity: 0,
        ttl_seconds: 0,
        status: DataExportOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_data_export_override("code-318", "name-841");
        assert_eq!(item.status, DataExportOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_data_export_override("code-318", "name-841");
        if item.move_to(DataExportOverrideStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportOverrideStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_data_export_override("code-318", "name-841");
        item.status = DataExportOverrideStatus::Cancelled;
        assert!(
            item.move_to(DataExportOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
