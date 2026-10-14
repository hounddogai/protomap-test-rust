use super::*;

use acme_kit::text;

/// Where a lenient measurement mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientMeasurementMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientMeasurementMappingStatus {
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

impl std::fmt::Display for LenientMeasurementMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient measurement mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementMapping {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub active: bool,
    pub channel: String,
    pub principal: String,
    pub version: i64,
    pub margin: f64,
    pub status: LenientMeasurementMappingStatus,
}

impl LenientMeasurementMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientMeasurementMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientMeasurementMappingStatus) -> bool {
        match self.status {
            LenientMeasurementMappingStatus::Open => {
                match next {
                    LenientMeasurementMappingStatus::InReview | LenientMeasurementMappingStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientMeasurementMappingStatus::InReview => {
                return next == LenientMeasurementMappingStatus::Resolved
                    || next == LenientMeasurementMappingStatus::Archived;
            }
            LenientMeasurementMappingStatus::Resolved => {
                match next {
                    LenientMeasurementMappingStatus::Reopened | LenientMeasurementMappingStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientMeasurementMappingStatus::Reopened => {
                return next == LenientMeasurementMappingStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientMeasurementMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient measurement mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.weight * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientMeasurementMapping {
        LenientMeasurementMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            active: self.active,
            channel: self.channel.clone(),
            principal: self.principal.clone(),
            version: self.version,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient measurement mapping in status open.
pub fn new_lenient_measurement_mapping(code: &str, label: &str) -> LenientMeasurementMapping {
    LenientMeasurementMapping {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        active: false,
        channel: "".to_string(),
        principal: "".to_string(),
        version: 0,
        margin: 0.0,
        status: LenientMeasurementMappingStatus::Open,
    }
}
