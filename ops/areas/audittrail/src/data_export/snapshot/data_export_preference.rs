use super::*;

use acme_kit::text;

/// Where a data export preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DataExportPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DataExportPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPreference {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub retries: i64,
    pub category: String,
    pub ends_at: i64,
    pub status: DataExportPreferenceStatus,
}

impl DataExportPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportPreferenceStatus) -> bool {
        match self.status {
            DataExportPreferenceStatus::Proposed => {
                return next == DataExportPreferenceStatus::Withdrawn;
            }
            DataExportPreferenceStatus::Approved => {
                match next {
                    DataExportPreferenceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportPreferenceStatus::Rejected => {
                match next {
                    DataExportPreferenceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DataExportPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.retries as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportPreference {
        DataExportPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            retries: self.retries,
            category: self.category.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a data export preference in status proposed.
pub fn new_data_export_preference(code: &str, name: &str) -> DataExportPreference {
    DataExportPreference {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        retries: 0,
        category: "".to_string(),
        ends_at: 0,
        status: DataExportPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_data_export_preference("code-410", "name-9");
        assert_eq!(item.status, DataExportPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_data_export_preference("code-410", "name-9");
        if item.move_to(DataExportPreferenceStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportPreferenceStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_data_export_preference("code-410", "name-9");
        item.status = DataExportPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(DataExportPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
