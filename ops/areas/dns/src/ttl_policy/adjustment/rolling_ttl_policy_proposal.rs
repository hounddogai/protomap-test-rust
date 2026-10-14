use super::*;

use acme_kit::text;

/// Where a rolling ttl policy proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTtlPolicyProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingTtlPolicyProposalStatus {
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

impl std::fmt::Display for RollingTtlPolicyProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling ttl policy proposal as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyProposal {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub contact_email: String,
    pub host: String,
    pub title: String,
    pub phone: String,
    pub duration_minutes: i64,
    pub limit: i64,
    pub version: i64,
    pub label: String,
    pub status: RollingTtlPolicyProposalStatus,
}

impl RollingTtlPolicyProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTtlPolicyProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTtlPolicyProposalStatus) -> bool {
        match self.status {
            RollingTtlPolicyProposalStatus::Idle => {
                return next == RollingTtlPolicyProposalStatus::Warming;
            }
            RollingTtlPolicyProposalStatus::Warming => {
                match next {
                    RollingTtlPolicyProposalStatus::Ready | RollingTtlPolicyProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTtlPolicyProposalStatus::Ready => {
                return next == RollingTtlPolicyProposalStatus::Draining;
            }
            RollingTtlPolicyProposalStatus::Draining => {
                return next == RollingTtlPolicyProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingTtlPolicyProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling ttl policy proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling ttl policy proposal", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.limit as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTtlPolicyProposal {
        RollingTtlPolicyProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            contact_email: self.contact_email.clone(),
            host: self.host.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            duration_minutes: self.duration_minutes,
            limit: self.limit,
            version: self.version,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling ttl policy proposal in status idle.
pub fn new_rolling_ttl_policy_proposal(code: &str, name: &str) -> RollingTtlPolicyProposal {
    RollingTtlPolicyProposal {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        contact_email: "".to_string(),
        host: "".to_string(),
        title: "".to_string(),
        phone: "".to_string(),
        duration_minutes: 0,
        limit: 0,
        version: 0,
        label: "".to_string(),
        status: RollingTtlPolicyProposalStatus::Idle,
    }
}
