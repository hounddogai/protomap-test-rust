use super::*;

use acme_kit::text;

/// Where a fallback measurement record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackMeasurementRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackMeasurementRecordStatus {
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

impl std::fmt::Display for FallbackMeasurementRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback measurement record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementRecord {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub segment: String,
    pub unit: String,
    pub ratio: f64,
    pub archived: bool,
    pub starts_at: i64,
    pub currency: String,
    pub quota_units: i64,
    pub status: FallbackMeasurementRecordStatus,
}

impl FallbackMeasurementRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackMeasurementRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackMeasurementRecordStatus) -> bool {
        match self.status {
            FallbackMeasurementRecordStatus::Open => {
                match next {
                    FallbackMeasurementRecordStatus::Reopened | FallbackMeasurementRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMeasurementRecordStatus::InReview => {
                match next {
                    FallbackMeasurementRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMeasurementRecordStatus::Resolved => {
                return next == FallbackMeasurementRecordStatus::Reopened;
            }
            FallbackMeasurementRecordStatus::Reopened => {
                return next == FallbackMeasurementRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackMeasurementRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback measurement record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.starts_at as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackMeasurementRecord {
        FallbackMeasurementRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            segment: self.segment.clone(),
            unit: self.unit.clone(),
            ratio: self.ratio,
            archived: self.archived,
            starts_at: self.starts_at,
            currency: self.currency.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback measurement record in status open.
pub fn new_fallback_measurement_record(code: &str, title: &str) -> FallbackMeasurementRecord {
    FallbackMeasurementRecord {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        segment: "".to_string(),
        unit: "".to_string(),
        ratio: 0.0,
        archived: false,
        starts_at: 0,
        currency: "".to_string(),
        quota_units: 0,
        status: FallbackMeasurementRecordStatus::Open,
    }
}
