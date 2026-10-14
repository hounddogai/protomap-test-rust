use super::*;

use acme_kit::text;

/// Where a flow log milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FlowLogMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FlowLogMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMilestone {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub locale: String,
    pub issuer: String,
    pub ceiling: f64,
    pub status: FlowLogMilestoneStatus,
}

impl FlowLogMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogMilestoneStatus) -> bool {
        match self.status {
            FlowLogMilestoneStatus::Requested => {
                match next {
                    FlowLogMilestoneStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogMilestoneStatus::Granted => {
                match next {
                    FlowLogMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogMilestoneStatus::Revoked => {
                match next {
                    FlowLogMilestoneStatus::Expired => {
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

    pub fn move_to(&mut self, next: FlowLogMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log milestone", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> FlowLogMilestone {
        FlowLogMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            locale: self.locale.clone(),
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a flow log milestone in status requested.
pub fn new_flow_log_milestone(code: &str, title: &str) -> FlowLogMilestone {
    FlowLogMilestone {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        locale: "".to_string(),
        issuer: "".to_string(),
        ceiling: 0.0,
        status: FlowLogMilestoneStatus::Requested,
    }
}
