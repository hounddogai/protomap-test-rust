use super::*;

use acme_kit::text;

/// Where a seasonal measurement manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalMeasurementManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalMeasurementManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalMeasurementManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal measurement manifest as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementManifest {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub visible: bool,
    pub label: String,
    pub amount: f64,
    pub status: SeasonalMeasurementManifestStatus,
}

impl SeasonalMeasurementManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalMeasurementManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalMeasurementManifestStatus) -> bool {
        match self.status {
            SeasonalMeasurementManifestStatus::Proposed => {
                return next == SeasonalMeasurementManifestStatus::Approved
                    || next == SeasonalMeasurementManifestStatus::Withdrawn;
            }
            SeasonalMeasurementManifestStatus::Approved => {
                match next {
                    SeasonalMeasurementManifestStatus::Rejected | SeasonalMeasurementManifestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalMeasurementManifestStatus::Rejected => {
                return next == SeasonalMeasurementManifestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalMeasurementManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal measurement manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal measurement manifest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> SeasonalMeasurementManifest {
        SeasonalMeasurementManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            visible: self.visible,
            label: self.label.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal measurement manifest in status proposed.
pub fn new_seasonal_measurement_manifest(code: &str, title: &str) -> SeasonalMeasurementManifest {
    SeasonalMeasurementManifest {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        visible: false,
        label: "".to_string(),
        amount: 0.0,
        status: SeasonalMeasurementManifestStatus::Proposed,
    }
}
