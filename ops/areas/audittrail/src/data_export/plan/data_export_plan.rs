use super::*;

use acme_kit::text;

/// Where a data export plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DataExportPlanStatus {
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

impl std::fmt::Display for DataExportPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export plan as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPlan {
    pub code: String,
    pub label: String,
    pub country: String,
    pub rate: f64,
    pub approved: bool,
    pub rotation_days: i64,
    pub region: String,
    pub port: i64,
    pub issuer: String,
    pub burst: i64,
    pub status: DataExportPlanStatus,
}

impl DataExportPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportPlanStatus) -> bool {
        match self.status {
            DataExportPlanStatus::Draft => {
                return next == DataExportPlanStatus::Pending || next == DataExportPlanStatus::Active;
            }
            DataExportPlanStatus::Pending => {
                match next {
                    DataExportPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportPlanStatus::Active => {
                match next {
                    DataExportPlanStatus::Suspended | DataExportPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportPlanStatus::Suspended => {
                match next {
                    DataExportPlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: DataExportPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.port as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportPlan {
        DataExportPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            rate: self.rate,
            approved: self.approved,
            rotation_days: self.rotation_days,
            region: self.region.clone(),
            port: self.port,
            issuer: self.issuer.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a data export plan in status draft.
pub fn new_data_export_plan(code: &str, label: &str) -> DataExportPlan {
    DataExportPlan {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        rate: 0.0,
        approved: false,
        rotation_days: 0,
        region: "".to_string(),
        port: 0,
        issuer: "".to_string(),
        burst: 0,
        status: DataExportPlanStatus::Draft,
    }
}
