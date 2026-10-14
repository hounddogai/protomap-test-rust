use super::*;

use acme_kit::text;

/// Where a data export rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DataExportRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DataExportRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRollup {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub region: String,
    pub checksum: String,
    pub issuer: String,
    pub summary: String,
    pub maximum: f64,
    pub status: DataExportRollupStatus,
}

impl DataExportRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportRollupStatus) -> bool {
        match self.status {
            DataExportRollupStatus::Proposed => {
                return next == DataExportRollupStatus::Approved || next == DataExportRollupStatus::Rejected;
            }
            DataExportRollupStatus::Approved => {
                return next == DataExportRollupStatus::Rejected || next == DataExportRollupStatus::Withdrawn;
            }
            DataExportRollupStatus::Rejected => {
                return next == DataExportRollupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.grace_days as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportRollup {
        DataExportRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            region: self.region.clone(),
            checksum: self.checksum.clone(),
            issuer: self.issuer.clone(),
            summary: self.summary.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a data export rollup in status proposed.
pub fn new_data_export_rollup(code: &str, title: &str) -> DataExportRollup {
    DataExportRollup {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        region: "".to_string(),
        checksum: "".to_string(),
        issuer: "".to_string(),
        summary: "".to_string(),
        maximum: 0.0,
        status: DataExportRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_data_export_rollup("code-225", "title-823");
        assert_eq!(item.status, DataExportRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_data_export_rollup("code-225", "title-823");
        if item.move_to(DataExportRollupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportRollupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_data_export_rollup("code-225", "title-823");
        item.status = DataExportRollupStatus::Withdrawn;
        assert!(
            item.move_to(DataExportRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
