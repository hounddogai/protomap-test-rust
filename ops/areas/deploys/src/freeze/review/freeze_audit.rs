use super::*;

use acme_kit::text;

/// Where a freeze audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FreezeAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FreezeAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAudit {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub region: String,
    pub reviewer: String,
    pub principal: String,
    pub quantity: i64,
    pub status: FreezeAuditStatus,
}

impl FreezeAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeAuditStatus) -> bool {
        match self.status {
            FreezeAuditStatus::Proposed => {
                match next {
                    FreezeAuditStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeAuditStatus::Approved => {
                return next == FreezeAuditStatus::Rejected;
            }
            FreezeAuditStatus::Rejected => {
                match next {
                    FreezeAuditStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FreezeAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> FreezeAudit {
        FreezeAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            region: self.region.clone(),
            reviewer: self.reviewer.clone(),
            principal: self.principal.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a freeze audit in status proposed.
pub fn new_freeze_audit(code: &str, name: &str) -> FreezeAudit {
    FreezeAudit {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        region: "".to_string(),
        reviewer: "".to_string(),
        principal: "".to_string(),
        quantity: 0,
        status: FreezeAuditStatus::Proposed,
    }
}
