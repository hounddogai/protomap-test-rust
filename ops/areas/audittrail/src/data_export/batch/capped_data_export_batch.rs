use super::*;

use acme_kit::text;

/// Where a capped data export batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDataExportBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedDataExportBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedDataExportBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped data export batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportBatch {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub weight: f64,
    pub issuer: String,
    pub batch_size: i64,
    pub priority: i64,
    pub scope: String,
    pub status: CappedDataExportBatchStatus,
}

impl CappedDataExportBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDataExportBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDataExportBatchStatus) -> bool {
        match self.status {
            CappedDataExportBatchStatus::Proposed => {
                return next == CappedDataExportBatchStatus::Rejected || next == CappedDataExportBatchStatus::Withdrawn;
            }
            CappedDataExportBatchStatus::Approved => {
                return next == CappedDataExportBatchStatus::Withdrawn;
            }
            CappedDataExportBatchStatus::Rejected => {
                match next {
                    CappedDataExportBatchStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CappedDataExportBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped data export batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped data export batch", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.batch_size as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDataExportBatch {
        CappedDataExportBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            weight: self.weight,
            issuer: self.issuer.clone(),
            batch_size: self.batch_size,
            priority: self.priority,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a capped data export batch in status proposed.
pub fn new_capped_data_export_batch(code: &str, title: &str) -> CappedDataExportBatch {
    CappedDataExportBatch {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        weight: 0.0,
        issuer: "".to_string(),
        batch_size: 0,
        priority: 0,
        scope: "".to_string(),
        status: CappedDataExportBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_data_export_batch("code-518", "title-387");
        assert_eq!(item.status, CappedDataExportBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_capped_data_export_batch("code-518", "title-387");
        if item.move_to(CappedDataExportBatchStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDataExportBatchStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_data_export_batch("code-518", "title-387");
        item.status = CappedDataExportBatchStatus::Withdrawn;
        assert!(
            item.move_to(CappedDataExportBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
