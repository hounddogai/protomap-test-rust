use super::*;

use acme_kit::text;

/// Where a strict data export metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDataExportMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictDataExportMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictDataExportMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict data export metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDataExportMetric {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub version: i64,
    pub rotation_days: i64,
    pub priority: i64,
    pub contact_email: String,
    pub sequence: i64,
    pub amount: f64,
    pub label: String,
    pub tenant_id: String,
    pub status: StrictDataExportMetricStatus,
}

impl StrictDataExportMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDataExportMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDataExportMetricStatus) -> bool {
        match self.status {
            StrictDataExportMetricStatus::Created => {
                return next == StrictDataExportMetricStatus::Validated
                    || next == StrictDataExportMetricStatus::Retired;
            }
            StrictDataExportMetricStatus::Validated => {
                match next {
                    StrictDataExportMetricStatus::Published | StrictDataExportMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDataExportMetricStatus::Published => {
                return next == StrictDataExportMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictDataExportMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict data export metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.amount + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictDataExportMetric {
        StrictDataExportMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            version: self.version,
            rotation_days: self.rotation_days,
            priority: self.priority,
            contact_email: self.contact_email.clone(),
            sequence: self.sequence,
            amount: self.amount,
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict data export metric in status created.
pub fn new_strict_data_export_metric(code: &str, title: &str) -> StrictDataExportMetric {
    StrictDataExportMetric {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        version: 0,
        rotation_days: 0,
        priority: 0,
        contact_email: "".to_string(),
        sequence: 0,
        amount: 0.0,
        label: "".to_string(),
        tenant_id: "".to_string(),
        status: StrictDataExportMetricStatus::Created,
    }
}
