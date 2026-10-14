use super::*;

use acme_kit::text;

/// Where a manual region pin bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRegionPinBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualRegionPinBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualRegionPinBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual region pin bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinBundle {
    pub code: String,
    pub label: String,
    pub name: String,
    pub priority: i64,
    pub external_ref: String,
    pub status: ManualRegionPinBundleStatus,
}

impl ManualRegionPinBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRegionPinBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRegionPinBundleStatus) -> bool {
        match self.status {
            ManualRegionPinBundleStatus::Proposed => {
                return next == ManualRegionPinBundleStatus::Approved || next == ManualRegionPinBundleStatus::Withdrawn;
            }
            ManualRegionPinBundleStatus::Approved => {
                match next {
                    ManualRegionPinBundleStatus::Rejected | ManualRegionPinBundleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRegionPinBundleStatus::Rejected => {
                match next {
                    ManualRegionPinBundleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualRegionPinBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual region pin bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> ManualRegionPinBundle {
        ManualRegionPinBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            priority: self.priority,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual region pin bundle in status proposed.
pub fn new_manual_region_pin_bundle(code: &str, label: &str) -> ManualRegionPinBundle {
    ManualRegionPinBundle {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        priority: 0,
        external_ref: "".to_string(),
        status: ManualRegionPinBundleStatus::Proposed,
    }
}
