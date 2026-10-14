use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rollout estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RolloutEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RolloutEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimate {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub sequence: i64,
    pub source: String,
    pub version: i64,
    pub notes: String,
    pub segment: String,
    pub minimum: f64,
    pub quantity: i64,
    pub status: RolloutEstimateStatus,
}

impl RolloutEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutEstimateStatus) -> bool {
        match self.status {
            RolloutEstimateStatus::Proposed => {
                return next == RolloutEstimateStatus::Withdrawn;
            }
            RolloutEstimateStatus::Approved => {
                match next {
                    RolloutEstimateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutEstimateStatus::Rejected => {
                match next {
                    RolloutEstimateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RolloutEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_load(&self) -> f64 {
        ((self.version as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutEstimate {
        RolloutEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            sequence: self.sequence,
            source: self.source.clone(),
            version: self.version,
            notes: self.notes.clone(),
            segment: self.segment.clone(),
            minimum: self.minimum,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rollout estimate in status proposed.
pub fn new_rollout_estimate(code: &str, name: &str) -> RolloutEstimate {
    RolloutEstimate {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        sequence: 0,
        source: "".to_string(),
        version: 0,
        notes: "".to_string(),
        segment: "".to_string(),
        minimum: 0.0,
        quantity: 0,
        status: RolloutEstimateStatus::Proposed,
    }
}
