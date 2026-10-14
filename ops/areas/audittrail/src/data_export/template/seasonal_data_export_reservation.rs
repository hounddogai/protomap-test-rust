use super::*;

use acme_kit::text;

/// Where a seasonal data export reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDataExportReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalDataExportReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SeasonalDataExportReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal data export reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportReservation {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub window_days: i64,
    pub batch_size: i64,
    pub status: SeasonalDataExportReservationStatus,
}

impl SeasonalDataExportReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDataExportReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDataExportReservationStatus) -> bool {
        match self.status {
            SeasonalDataExportReservationStatus::New => {
                return next == SeasonalDataExportReservationStatus::Done;
            }
            SeasonalDataExportReservationStatus::Queued => {
                return next == SeasonalDataExportReservationStatus::Running
                    || next == SeasonalDataExportReservationStatus::Done;
            }
            SeasonalDataExportReservationStatus::Running => {
                match next {
                    SeasonalDataExportReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDataExportReservationStatus::Done => {
                match next {
                    SeasonalDataExportReservationStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalDataExportReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal data export reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.batch_size as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDataExportReservation {
        SeasonalDataExportReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            window_days: self.window_days,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal data export reservation in status new.
pub fn new_seasonal_data_export_reservation(code: &str, label: &str) -> SeasonalDataExportReservation {
    SeasonalDataExportReservation {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        window_days: 0,
        batch_size: 0,
        status: SeasonalDataExportReservationStatus::New,
    }
}
