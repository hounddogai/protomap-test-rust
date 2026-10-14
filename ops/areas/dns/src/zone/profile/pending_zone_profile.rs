use super::*;

use acme_kit::text;

/// Where a pending zone profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingZoneProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingZoneProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingZoneProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending zone profile as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneProfile {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub batch_size: i64,
    pub rate: f64,
    pub quota_units: i64,
    pub owner_id: String,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub label: String,
    pub maximum: f64,
    pub status: PendingZoneProfileStatus,
}

impl PendingZoneProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingZoneProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingZoneProfileStatus) -> bool {
        match self.status {
            PendingZoneProfileStatus::Created => {
                return next == PendingZoneProfileStatus::Published || next == PendingZoneProfileStatus::Retired;
            }
            PendingZoneProfileStatus::Validated => {
                match next {
                    PendingZoneProfileStatus::Published | PendingZoneProfileStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingZoneProfileStatus::Published => {
                match next {
                    PendingZoneProfileStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingZoneProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending zone profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending zone profile", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.duration_minutes as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingZoneProfile {
        PendingZoneProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            batch_size: self.batch_size,
            rate: self.rate,
            quota_units: self.quota_units,
            owner_id: self.owner_id.clone(),
            duration_minutes: self.duration_minutes,
            tags: self.tags.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending zone profile in status created.
pub fn new_pending_zone_profile(code: &str, name: &str) -> PendingZoneProfile {
    PendingZoneProfile {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        batch_size: 0,
        rate: 0.0,
        quota_units: 0,
        owner_id: "".to_string(),
        duration_minutes: 0,
        tags: Vec::new(),
        label: "".to_string(),
        maximum: 0.0,
        status: PendingZoneProfileStatus::Created,
    }
}
