use super::*;

use acme_kit::text;

/// Where a data export mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DataExportMappingStatus {
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

impl std::fmt::Display for DataExportMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export mapping as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMapping {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub approved: bool,
    pub status: DataExportMappingStatus,
}

impl DataExportMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportMappingStatus) -> bool {
        match self.status {
            DataExportMappingStatus::Planned => {
                return next == DataExportMappingStatus::Completed;
            }
            DataExportMappingStatus::Scheduled => {
                match next {
                    DataExportMappingStatus::InProgress | DataExportMappingStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportMappingStatus::InProgress => {
                match next {
                    DataExportMappingStatus::Completed | DataExportMappingStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportMappingStatus::Completed => {
                return next == DataExportMappingStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> DataExportMapping {
        DataExportMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a data export mapping in status planned.
pub fn new_data_export_mapping(code: &str, label: &str) -> DataExportMapping {
    DataExportMapping {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        tags: Vec::new(),
        approved: false,
        status: DataExportMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_data_export_mapping("code-304", "label-36");
        assert_eq!(item.status, DataExportMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_data_export_mapping("code-304", "label-36");
        if item.move_to(DataExportMappingStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportMappingStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_data_export_mapping("code-304", "label-36");
        item.status = DataExportMappingStatus::Cancelled;
        assert!(
            item.move_to(DataExportMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
