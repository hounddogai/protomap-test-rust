use super::*;

use acme_kit::text;

/// Where a region pin allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionPinAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionPinAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAllocation {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub checksum: String,
    pub category: String,
    pub description: String,
    pub host: String,
    pub visible: bool,
    pub duration_minutes: i64,
    pub locale: String,
    pub status: RegionPinAllocationStatus,
}

impl RegionPinAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinAllocationStatus) -> bool {
        match self.status {
            RegionPinAllocationStatus::Requested => {
                return next == RegionPinAllocationStatus::Revoked || next == RegionPinAllocationStatus::Expired;
            }
            RegionPinAllocationStatus::Granted => {
                return next == RegionPinAllocationStatus::Revoked || next == RegionPinAllocationStatus::Expired;
            }
            RegionPinAllocationStatus::Revoked => {
                match next {
                    RegionPinAllocationStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionPinAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.rotation_days as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinAllocation {
        RegionPinAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            checksum: self.checksum.clone(),
            category: self.category.clone(),
            description: self.description.clone(),
            host: self.host.clone(),
            visible: self.visible,
            duration_minutes: self.duration_minutes,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a region pin allocation in status requested.
pub fn new_region_pin_allocation(code: &str, title: &str) -> RegionPinAllocation {
    RegionPinAllocation {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        checksum: "".to_string(),
        category: "".to_string(),
        description: "".to_string(),
        host: "".to_string(),
        visible: false,
        duration_minutes: 0,
        locale: "".to_string(),
        status: RegionPinAllocationStatus::Requested,
    }
}
