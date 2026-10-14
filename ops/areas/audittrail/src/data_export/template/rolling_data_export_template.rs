use super::*;

use acme_kit::text;

/// Where a rolling data export template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDataExportTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingDataExportTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RollingDataExportTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling data export template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportTemplate {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub ends_at: i64,
    pub pinned: bool,
    pub visible: bool,
    pub reviewer: String,
    pub country: String,
    pub approved: bool,
    pub status: RollingDataExportTemplateStatus,
}

impl RollingDataExportTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDataExportTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDataExportTemplateStatus) -> bool {
        match self.status {
            RollingDataExportTemplateStatus::Open => {
                return next == RollingDataExportTemplateStatus::Resolved
                    || next == RollingDataExportTemplateStatus::Archived;
            }
            RollingDataExportTemplateStatus::InReview => {
                match next {
                    RollingDataExportTemplateStatus::Resolved | RollingDataExportTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDataExportTemplateStatus::Resolved => {
                match next {
                    RollingDataExportTemplateStatus::Reopened | RollingDataExportTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDataExportTemplateStatus::Reopened => {
                match next {
                    RollingDataExportTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingDataExportTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling data export template {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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
        self.ends_at as f64
    }

    pub fn copy(&self) -> RollingDataExportTemplate {
        RollingDataExportTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            ends_at: self.ends_at,
            pinned: self.pinned,
            visible: self.visible,
            reviewer: self.reviewer.clone(),
            country: self.country.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling data export template in status open.
pub fn new_rolling_data_export_template(code: &str, label: &str) -> RollingDataExportTemplate {
    RollingDataExportTemplate {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        ends_at: 0,
        pinned: false,
        visible: false,
        reviewer: "".to_string(),
        country: "".to_string(),
        approved: false,
        status: RollingDataExportTemplateStatus::Open,
    }
}
