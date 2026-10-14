use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional zone override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalZoneOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalZoneOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RegionalZoneOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional zone override as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverride {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub quota_units: i64,
    pub duration_minutes: i64,
    pub revision: i64,
    pub unit: String,
    pub floor_value: f64,
    pub limit: i64,
    pub version: i64,
    pub host: String,
    pub status: RegionalZoneOverrideStatus,
}

impl RegionalZoneOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalZoneOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalZoneOverrideStatus) -> bool {
        match self.status {
            RegionalZoneOverrideStatus::Draft => {
                return next == RegionalZoneOverrideStatus::Suspended;
            }
            RegionalZoneOverrideStatus::Pending => {
                match next {
                    RegionalZoneOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalZoneOverrideStatus::Active => {
                match next {
                    RegionalZoneOverrideStatus::Suspended | RegionalZoneOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalZoneOverrideStatus::Suspended => {
                return next == RegionalZoneOverrideStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalZoneOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional zone override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional zone override", self.code)
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

    pub fn projected_exposure(&self) -> f64 {
        (self.floor_value * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalZoneOverride {
        RegionalZoneOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            quota_units: self.quota_units,
            duration_minutes: self.duration_minutes,
            revision: self.revision,
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            version: self.version,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a regional zone override in status draft.
pub fn new_regional_zone_override(code: &str, label: &str) -> RegionalZoneOverride {
    RegionalZoneOverride {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        quota_units: 0,
        duration_minutes: 0,
        revision: 0,
        unit: "".to_string(),
        floor_value: 0.0,
        limit: 0,
        version: 0,
        host: "".to_string(),
        status: RegionalZoneOverrideStatus::Draft,
    }
}
