use super::*;

use acme_kit::text;

/// Where a region pin manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionPinManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionPinManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinManifest {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub checksum: String,
    pub floor_value: f64,
    pub status: RegionPinManifestStatus,
}

impl RegionPinManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinManifestStatus) -> bool {
        match self.status {
            RegionPinManifestStatus::Created => {
                match next {
                    RegionPinManifestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinManifestStatus::Validated => {
                return next == RegionPinManifestStatus::Retired;
            }
            RegionPinManifestStatus::Published => {
                return next == RegionPinManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.maximum + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinManifest {
        RegionPinManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            checksum: self.checksum.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a region pin manifest in status created.
pub fn new_region_pin_manifest(code: &str, label: &str) -> RegionPinManifest {
    RegionPinManifest {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        checksum: "".to_string(),
        floor_value: 0.0,
        status: RegionPinManifestStatus::Created,
    }
}
