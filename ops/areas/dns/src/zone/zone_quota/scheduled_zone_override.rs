use super::*;

use acme_kit::text;

/// Where a scheduled zone override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledZoneOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledZoneOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledZoneOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled zone override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneOverride {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub rotation_days: i64,
    pub host: String,
    pub revision: i64,
    pub status: ScheduledZoneOverrideStatus,
}

impl ScheduledZoneOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledZoneOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledZoneOverrideStatus) -> bool {
        match self.status {
            ScheduledZoneOverrideStatus::Proposed => {
                return next == ScheduledZoneOverrideStatus::Approved || next == ScheduledZoneOverrideStatus::Withdrawn;
            }
            ScheduledZoneOverrideStatus::Approved => {
                return next == ScheduledZoneOverrideStatus::Rejected;
            }
            ScheduledZoneOverrideStatus::Rejected => {
                match next {
                    ScheduledZoneOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledZoneOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled zone override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.rotation_days as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledZoneOverride {
        ScheduledZoneOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            rotation_days: self.rotation_days,
            host: self.host.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled zone override in status proposed.
pub fn new_scheduled_zone_override(code: &str, name: &str) -> ScheduledZoneOverride {
    ScheduledZoneOverride {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        rotation_days: 0,
        host: "".to_string(),
        revision: 0,
        status: ScheduledZoneOverrideStatus::Proposed,
    }
}
