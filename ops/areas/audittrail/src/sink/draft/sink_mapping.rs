use super::*;

use acme_kit::text;

/// Where a sink mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SinkMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SinkMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkMapping {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub maximum: f64,
    pub ratio: f64,
    pub rotation_days: i64,
    pub rate: f64,
    pub score: f64,
    pub grace_days: i64,
    pub notes: String,
    pub minimum: f64,
    pub status: SinkMappingStatus,
}

impl SinkMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkMappingStatus) -> bool {
        match self.status {
            SinkMappingStatus::Proposed => {
                return next == SinkMappingStatus::Rejected;
            }
            SinkMappingStatus::Approved => {
                match next {
                    SinkMappingStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkMappingStatus::Rejected => {
                return next == SinkMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink mapping", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.maximum - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkMapping {
        SinkMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            maximum: self.maximum,
            ratio: self.ratio,
            rotation_days: self.rotation_days,
            rate: self.rate,
            score: self.score,
            grace_days: self.grace_days,
            notes: self.notes.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a sink mapping in status proposed.
pub fn new_sink_mapping(code: &str, name: &str) -> SinkMapping {
    SinkMapping {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        maximum: 0.0,
        ratio: 0.0,
        rotation_days: 0,
        rate: 0.0,
        score: 0.0,
        grace_days: 0,
        notes: "".to_string(),
        minimum: 0.0,
        status: SinkMappingStatus::Proposed,
    }
}
