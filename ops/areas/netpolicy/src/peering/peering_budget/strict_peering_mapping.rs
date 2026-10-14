use super::*;

use acme_kit::text;

/// Where a strict peering mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPeeringMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictPeeringMappingStatus {
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

impl std::fmt::Display for StrictPeeringMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict peering mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringMapping {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub country: String,
    pub port: i64,
    pub external_ref: String,
    pub maximum: f64,
    pub ttl_seconds: i64,
    pub status: StrictPeeringMappingStatus,
}

impl StrictPeeringMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPeeringMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPeeringMappingStatus) -> bool {
        match self.status {
            StrictPeeringMappingStatus::Open => {
                return next == StrictPeeringMappingStatus::InReview || next == StrictPeeringMappingStatus::Reopened;
            }
            StrictPeeringMappingStatus::InReview => {
                match next {
                    StrictPeeringMappingStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPeeringMappingStatus::Resolved => {
                match next {
                    StrictPeeringMappingStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPeeringMappingStatus::Reopened => {
                return next == StrictPeeringMappingStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPeeringMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict peering mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPeeringMapping {
        StrictPeeringMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            country: self.country.clone(),
            port: self.port,
            external_ref: self.external_ref.clone(),
            maximum: self.maximum,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict peering mapping in status open.
pub fn new_strict_peering_mapping(code: &str, label: &str) -> StrictPeeringMapping {
    StrictPeeringMapping {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        country: "".to_string(),
        port: 0,
        external_ref: "".to_string(),
        maximum: 0.0,
        ttl_seconds: 0,
        status: StrictPeeringMappingStatus::Open,
    }
}
