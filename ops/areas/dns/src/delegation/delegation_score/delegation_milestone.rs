use super::*;

use acme_kit::text;

/// Where a delegation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DelegationMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DelegationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMilestone {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub active: bool,
    pub priority: i64,
    pub key_id: String,
    pub minimum: f64,
    pub discount: f64,
    pub checksum: String,
    pub status: DelegationMilestoneStatus,
}

impl DelegationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationMilestoneStatus) -> bool {
        match self.status {
            DelegationMilestoneStatus::Idle => {
                match next {
                    DelegationMilestoneStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationMilestoneStatus::Warming => {
                return next == DelegationMilestoneStatus::Ready;
            }
            DelegationMilestoneStatus::Ready => {
                match next {
                    DelegationMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationMilestoneStatus::Draining => {
                return next == DelegationMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.discount + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationMilestone {
        DelegationMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            active: self.active,
            priority: self.priority,
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            discount: self.discount,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a delegation milestone in status idle.
pub fn new_delegation_milestone(code: &str, name: &str) -> DelegationMilestone {
    DelegationMilestone {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        active: false,
        priority: 0,
        key_id: "".to_string(),
        minimum: 0.0,
        discount: 0.0,
        checksum: "".to_string(),
        status: DelegationMilestoneStatus::Idle,
    }
}
