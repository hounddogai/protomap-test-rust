use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling usage manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingUsageManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingUsageManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingUsageManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling usage manifest as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageManifest {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub country: String,
    pub weight: f64,
    pub segment: String,
    pub active: bool,
    pub status: RollingUsageManifestStatus,
}

impl RollingUsageManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingUsageManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingUsageManifestStatus) -> bool {
        match self.status {
            RollingUsageManifestStatus::Proposed => {
                match next {
                    RollingUsageManifestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageManifestStatus::Approved => {
                match next {
                    RollingUsageManifestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageManifestStatus::Rejected => {
                return next == RollingUsageManifestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingUsageManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling usage manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> RollingUsageManifest {
        RollingUsageManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            country: self.country.clone(),
            weight: self.weight,
            segment: self.segment.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling usage manifest in status proposed.
pub fn new_rolling_usage_manifest(code: &str, title: &str) -> RollingUsageManifest {
    RollingUsageManifest {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        country: "".to_string(),
        weight: 0.0,
        segment: "".to_string(),
        active: false,
        status: RollingUsageManifestStatus::Proposed,
    }
}
