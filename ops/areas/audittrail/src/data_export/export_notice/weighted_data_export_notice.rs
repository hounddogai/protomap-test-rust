use super::*;

use acme_kit::text;

/// Where a weighted data export notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedDataExportNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedDataExportNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for WeightedDataExportNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted data export notice as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportNotice {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub phone: String,
    pub floor_value: f64,
    pub country: String,
    pub burst: i64,
    pub threshold: f64,
    pub margin: f64,
    pub status: WeightedDataExportNoticeStatus,
}

impl WeightedDataExportNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedDataExportNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedDataExportNoticeStatus) -> bool {
        match self.status {
            WeightedDataExportNoticeStatus::Planned => {
                return next == WeightedDataExportNoticeStatus::Scheduled
                    || next == WeightedDataExportNoticeStatus::InProgress;
            }
            WeightedDataExportNoticeStatus::Scheduled => {
                return next == WeightedDataExportNoticeStatus::Cancelled;
            }
            WeightedDataExportNoticeStatus::InProgress => {
                match next {
                    WeightedDataExportNoticeStatus::Completed | WeightedDataExportNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedDataExportNoticeStatus::Completed => {
                return next == WeightedDataExportNoticeStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedDataExportNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted data export notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted data export notice", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.threshold * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedDataExportNotice {
        WeightedDataExportNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            phone: self.phone.clone(),
            floor_value: self.floor_value,
            country: self.country.clone(),
            burst: self.burst,
            threshold: self.threshold,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted data export notice in status planned.
pub fn new_weighted_data_export_notice(code: &str, name: &str) -> WeightedDataExportNotice {
    WeightedDataExportNotice {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        phone: "".to_string(),
        floor_value: 0.0,
        country: "".to_string(),
        burst: 0,
        threshold: 0.0,
        margin: 0.0,
        status: WeightedDataExportNoticeStatus::Planned,
    }
}
