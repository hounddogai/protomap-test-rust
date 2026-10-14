use super::*;

use acme_kit::text;

/// Where a data export metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DataExportMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DataExportMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMetric {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub cost: f64,
    pub source: String,
    pub weight: f64,
    pub contact_email: String,
    pub external_ref: String,
    pub status: DataExportMetricStatus,
}

impl DataExportMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportMetricStatus) -> bool {
        match self.status {
            DataExportMetricStatus::Proposed => {
                match next {
                    DataExportMetricStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportMetricStatus::Approved => {
                return next == DataExportMetricStatus::Rejected || next == DataExportMetricStatus::Withdrawn;
            }
            DataExportMetricStatus::Rejected => {
                match next {
                    DataExportMetricStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DataExportMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.margin * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportMetric {
        DataExportMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            cost: self.cost,
            source: self.source.clone(),
            weight: self.weight,
            contact_email: self.contact_email.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a data export metric in status proposed.
pub fn new_data_export_metric(code: &str, label: &str) -> DataExportMetric {
    DataExportMetric {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        cost: 0.0,
        source: "".to_string(),
        weight: 0.0,
        contact_email: "".to_string(),
        external_ref: "".to_string(),
        status: DataExportMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_data_export_metric("code-245", "label-622");
        assert_eq!(item.status, DataExportMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_data_export_metric("code-245", "label-622");
        if item.move_to(DataExportMetricStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportMetricStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_data_export_metric("code-245", "label-622");
        item.status = DataExportMetricStatus::Withdrawn;
        assert!(
            item.move_to(DataExportMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
