use super::*;

use acme_kit::text;

/// Where a data export template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DataExportTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DataExportTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTemplate {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub external_ref: String,
    pub approved: bool,
    pub capacity: i64,
    pub ends_at: i64,
    pub quantity: i64,
    pub status: DataExportTemplateStatus,
}

impl DataExportTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportTemplateStatus) -> bool {
        match self.status {
            DataExportTemplateStatus::Proposed => {
                match next {
                    DataExportTemplateStatus::Approved | DataExportTemplateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportTemplateStatus::Approved => {
                match next {
                    DataExportTemplateStatus::Rejected | DataExportTemplateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportTemplateStatus::Rejected => {
                match next {
                    DataExportTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DataExportTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export template {} cannot move from {} to {}",
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
        format!("{} {}", "data export template", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quantity as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportTemplate {
        DataExportTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            capacity: self.capacity,
            ends_at: self.ends_at,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a data export template in status proposed.
pub fn new_data_export_template(code: &str, name: &str) -> DataExportTemplate {
    DataExportTemplate {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        external_ref: "".to_string(),
        approved: false,
        capacity: 0,
        ends_at: 0,
        quantity: 0,
        status: DataExportTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_data_export_template("code-684", "name-117");
        assert_eq!(item.status, DataExportTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_data_export_template("code-684", "name-117");
        if item.move_to(DataExportTemplateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportTemplateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_data_export_template("code-684", "name-117");
        item.status = DataExportTemplateStatus::Withdrawn;
        assert!(
            item.move_to(DataExportTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
