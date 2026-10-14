use super::*;

use acme_kit::text;

/// Where a data export summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DataExportSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DataExportSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSummary {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub maximum: f64,
    pub revision: i64,
    pub created_at: i64,
    pub status: DataExportSummaryStatus,
}

impl DataExportSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportSummaryStatus) -> bool {
        match self.status {
            DataExportSummaryStatus::Proposed => {
                return next == DataExportSummaryStatus::Rejected || next == DataExportSummaryStatus::Withdrawn;
            }
            DataExportSummaryStatus::Approved => {
                match next {
                    DataExportSummaryStatus::Rejected | DataExportSummaryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportSummaryStatus::Rejected => {
                return next == DataExportSummaryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export summary", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.revision as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportSummary {
        DataExportSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            maximum: self.maximum,
            revision: self.revision,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a data export summary in status proposed.
pub fn new_data_export_summary(code: &str, name: &str) -> DataExportSummary {
    DataExportSummary {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        maximum: 0.0,
        revision: 0,
        created_at: 0,
        status: DataExportSummaryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_data_export_summary("code-88", "name-527");
        assert_eq!(item.status, DataExportSummaryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_data_export_summary("code-88", "name-527");
        if item.move_to(DataExportSummaryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportSummaryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_data_export_summary("code-88", "name-527");
        item.status = DataExportSummaryStatus::Withdrawn;
        assert!(
            item.move_to(DataExportSummaryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
