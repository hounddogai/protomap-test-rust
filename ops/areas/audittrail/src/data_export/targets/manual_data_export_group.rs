use super::*;

use acme_kit::text;

/// Where a manual data export group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDataExportGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualDataExportGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualDataExportGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual data export group as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportGroup {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub weight: f64,
    pub region: String,
    pub rate: f64,
    pub cluster: String,
    pub ratio: f64,
    pub notes: String,
    pub locale: String,
    pub status: ManualDataExportGroupStatus,
}

impl ManualDataExportGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDataExportGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDataExportGroupStatus) -> bool {
        match self.status {
            ManualDataExportGroupStatus::Proposed => {
                match next {
                    ManualDataExportGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportGroupStatus::Approved => {
                match next {
                    ManualDataExportGroupStatus::Rejected | ManualDataExportGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportGroupStatus::Rejected => {
                return next == ManualDataExportGroupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualDataExportGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual data export group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.capacity as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDataExportGroup {
        ManualDataExportGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            weight: self.weight,
            region: self.region.clone(),
            rate: self.rate,
            cluster: self.cluster.clone(),
            ratio: self.ratio,
            notes: self.notes.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual data export group in status proposed.
pub fn new_manual_data_export_group(code: &str, title: &str) -> ManualDataExportGroup {
    ManualDataExportGroup {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        weight: 0.0,
        region: "".to_string(),
        rate: 0.0,
        cluster: "".to_string(),
        ratio: 0.0,
        notes: "".to_string(),
        locale: "".to_string(),
        status: ManualDataExportGroupStatus::Proposed,
    }
}
