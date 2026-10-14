use super::*;

use acme_kit::text;

/// Where a data export entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DataExportEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for DataExportEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntry {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub phone: String,
    pub ratio: f64,
    pub status: DataExportEntryStatus,
}

impl DataExportEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportEntryStatus) -> bool {
        match self.status {
            DataExportEntryStatus::Draft => {
                return next == DataExportEntryStatus::Suspended;
            }
            DataExportEntryStatus::Pending => {
                return next == DataExportEntryStatus::Active || next == DataExportEntryStatus::Suspended;
            }
            DataExportEntryStatus::Active => {
                return next == DataExportEntryStatus::Suspended || next == DataExportEntryStatus::Closed;
            }
            DataExportEntryStatus::Suspended => {
                return next == DataExportEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export entry {} cannot move from {} to {}",
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
        format!("{} {}", "data export entry", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ratio + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportEntry {
        DataExportEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            phone: self.phone.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a data export entry in status draft.
pub fn new_data_export_entry(code: &str, name: &str) -> DataExportEntry {
    DataExportEntry {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        phone: "".to_string(),
        ratio: 0.0,
        status: DataExportEntryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_data_export_entry("code-794", "name-524");
        assert_eq!(item.status, DataExportEntryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_data_export_entry("code-794", "name-524");
        if item.move_to(DataExportEntryStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportEntryStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_data_export_entry("code-794", "name-524");
        item.status = DataExportEntryStatus::Closed;
        assert!(
            item.move_to(DataExportEntryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
