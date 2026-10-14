use super::*;

use acme_kit::text;

/// Where a scheduled data export template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDataExportTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledDataExportTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledDataExportTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled data export template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportTemplate {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub principal: String,
    pub summary: String,
    pub phone: String,
    pub pinned: bool,
    pub name: String,
    pub region: String,
    pub limit: i64,
    pub status: ScheduledDataExportTemplateStatus,
}

impl ScheduledDataExportTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDataExportTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDataExportTemplateStatus) -> bool {
        match self.status {
            ScheduledDataExportTemplateStatus::Created => {
                match next {
                    ScheduledDataExportTemplateStatus::Validated | ScheduledDataExportTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDataExportTemplateStatus::Validated => {
                return next == ScheduledDataExportTemplateStatus::Published
                    || next == ScheduledDataExportTemplateStatus::Retired;
            }
            ScheduledDataExportTemplateStatus::Published => {
                match next {
                    ScheduledDataExportTemplateStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledDataExportTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled data export template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> ScheduledDataExportTemplate {
        ScheduledDataExportTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            principal: self.principal.clone(),
            summary: self.summary.clone(),
            phone: self.phone.clone(),
            pinned: self.pinned,
            name: self.name.clone(),
            region: self.region.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a scheduled data export template in status created.
pub fn new_scheduled_data_export_template(code: &str, label: &str) -> ScheduledDataExportTemplate {
    ScheduledDataExportTemplate {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        principal: "".to_string(),
        summary: "".to_string(),
        phone: "".to_string(),
        pinned: false,
        name: "".to_string(),
        region: "".to_string(),
        limit: 0,
        status: ScheduledDataExportTemplateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_data_export_template("code-737", "label-283");
        assert_eq!(item.status, ScheduledDataExportTemplateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_scheduled_data_export_template("code-737", "label-283");
        if item.move_to(ScheduledDataExportTemplateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDataExportTemplateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_data_export_template("code-737", "label-283");
        item.status = ScheduledDataExportTemplateStatus::Retired;
        assert!(
            item.move_to(ScheduledDataExportTemplateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
