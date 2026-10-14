use super::*;

use acme_kit::text;

/// Where a data export notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DataExportNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DataExportNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportNotice {
    pub code: String,
    pub label: String,
    pub country: String,
    pub principal: String,
    pub tenant_id: String,
    pub summary: String,
    pub scope: String,
    pub amount: f64,
    pub status: DataExportNoticeStatus,
}

impl DataExportNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportNoticeStatus) -> bool {
        match self.status {
            DataExportNoticeStatus::Proposed => {
                return next == DataExportNoticeStatus::Withdrawn;
            }
            DataExportNoticeStatus::Approved => {
                match next {
                    DataExportNoticeStatus::Rejected | DataExportNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportNoticeStatus::Rejected => {
                return next == DataExportNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "data export notice", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> DataExportNotice {
        DataExportNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            principal: self.principal.clone(),
            tenant_id: self.tenant_id.clone(),
            summary: self.summary.clone(),
            scope: self.scope.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a data export notice in status proposed.
pub fn new_data_export_notice(code: &str, label: &str) -> DataExportNotice {
    DataExportNotice {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        principal: "".to_string(),
        tenant_id: "".to_string(),
        summary: "".to_string(),
        scope: "".to_string(),
        amount: 0.0,
        status: DataExportNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_data_export_notice("code-172", "label-498");
        assert_eq!(item.status, DataExportNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_data_export_notice("code-172", "label-498");
        if item.move_to(DataExportNoticeStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportNoticeStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_data_export_notice("code-172", "label-498");
        item.status = DataExportNoticeStatus::Withdrawn;
        assert!(
            item.move_to(DataExportNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
