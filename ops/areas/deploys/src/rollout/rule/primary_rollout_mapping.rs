use super::*;

use acme_kit::text;

/// Where a primary rollout mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRolloutMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryRolloutMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryRolloutMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary rollout mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutMapping {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub ratio: f64,
    pub principal: String,
    pub checksum: String,
    pub audience: String,
    pub approved: bool,
    pub description: String,
    pub visible: bool,
    pub status: PrimaryRolloutMappingStatus,
}

impl PrimaryRolloutMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRolloutMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRolloutMappingStatus) -> bool {
        match self.status {
            PrimaryRolloutMappingStatus::Requested => {
                return next == PrimaryRolloutMappingStatus::Granted;
            }
            PrimaryRolloutMappingStatus::Granted => {
                match next {
                    PrimaryRolloutMappingStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRolloutMappingStatus::Revoked => {
                match next {
                    PrimaryRolloutMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryRolloutMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rollout mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.rotation_days as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRolloutMapping {
        PrimaryRolloutMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            ratio: self.ratio,
            principal: self.principal.clone(),
            checksum: self.checksum.clone(),
            audience: self.audience.clone(),
            approved: self.approved,
            description: self.description.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary rollout mapping in status requested.
pub fn new_primary_rollout_mapping(code: &str, label: &str) -> PrimaryRolloutMapping {
    PrimaryRolloutMapping {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        ratio: 0.0,
        principal: "".to_string(),
        checksum: "".to_string(),
        audience: "".to_string(),
        approved: false,
        description: "".to_string(),
        visible: false,
        status: PrimaryRolloutMappingStatus::Requested,
    }
}
