use super::*;

use acme_kit::text;

/// Where a legacy federation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFederationAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyFederationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LegacyFederationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy federation allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocation {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub discount: f64,
    pub cipher: String,
    pub visible: bool,
    pub status: LegacyFederationAllocationStatus,
}

impl LegacyFederationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFederationAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFederationAllocationStatus) -> bool {
        match self.status {
            LegacyFederationAllocationStatus::Planned => {
                return next == LegacyFederationAllocationStatus::InProgress;
            }
            LegacyFederationAllocationStatus::Scheduled => {
                return next == LegacyFederationAllocationStatus::Cancelled;
            }
            LegacyFederationAllocationStatus::InProgress => {
                match next {
                    LegacyFederationAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFederationAllocationStatus::Completed => {
                match next {
                    LegacyFederationAllocationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LegacyFederationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy federation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> LegacyFederationAllocation {
        LegacyFederationAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            discount: self.discount,
            cipher: self.cipher.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy federation allocation in status planned.
pub fn new_legacy_federation_allocation(code: &str, name: &str) -> LegacyFederationAllocation {
    LegacyFederationAllocation {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        discount: 0.0,
        cipher: "".to_string(),
        visible: false,
        status: LegacyFederationAllocationStatus::Planned,
    }
}
