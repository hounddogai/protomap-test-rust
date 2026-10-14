use super::*;

use acme_kit::text;

/// Where a tiered suspension profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSuspensionProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredSuspensionProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredSuspensionProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered suspension profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionProfile {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub rotation_days: i64,
    pub rate: f64,
    pub tenant_id: String,
    pub status: TieredSuspensionProfileStatus,
}

impl TieredSuspensionProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSuspensionProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSuspensionProfileStatus) -> bool {
        match self.status {
            TieredSuspensionProfileStatus::Proposed => {
                return next == TieredSuspensionProfileStatus::Rejected;
            }
            TieredSuspensionProfileStatus::Approved => {
                return next == TieredSuspensionProfileStatus::Withdrawn;
            }
            TieredSuspensionProfileStatus::Rejected => {
                match next {
                    TieredSuspensionProfileStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredSuspensionProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered suspension profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.rate - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSuspensionProfile {
        TieredSuspensionProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            rotation_days: self.rotation_days,
            rate: self.rate,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered suspension profile in status proposed.
pub fn new_tiered_suspension_profile(code: &str, name: &str) -> TieredSuspensionProfile {
    TieredSuspensionProfile {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        rotation_days: 0,
        rate: 0.0,
        tenant_id: "".to_string(),
        status: TieredSuspensionProfileStatus::Proposed,
    }
}
