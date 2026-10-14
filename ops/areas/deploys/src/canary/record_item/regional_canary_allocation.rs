use super::*;

use acme_kit::text;

/// Where a regional canary allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalCanaryAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalCanaryAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalCanaryAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional canary allocation as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAllocation {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub channel: String,
    pub scope: String,
    pub visible: bool,
    pub maximum: f64,
    pub priority: i64,
    pub burst: i64,
    pub issuer: String,
    pub status: RegionalCanaryAllocationStatus,
}

impl RegionalCanaryAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalCanaryAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalCanaryAllocationStatus) -> bool {
        match self.status {
            RegionalCanaryAllocationStatus::Requested => {
                return next == RegionalCanaryAllocationStatus::Granted
                    || next == RegionalCanaryAllocationStatus::Expired;
            }
            RegionalCanaryAllocationStatus::Granted => {
                match next {
                    RegionalCanaryAllocationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalCanaryAllocationStatus::Revoked => {
                return next == RegionalCanaryAllocationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalCanaryAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional canary allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.maximum - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalCanaryAllocation {
        RegionalCanaryAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            channel: self.channel.clone(),
            scope: self.scope.clone(),
            visible: self.visible,
            maximum: self.maximum,
            priority: self.priority,
            burst: self.burst,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional canary allocation in status requested.
pub fn new_regional_canary_allocation(code: &str, name: &str) -> RegionalCanaryAllocation {
    RegionalCanaryAllocation {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        channel: "".to_string(),
        scope: "".to_string(),
        visible: false,
        maximum: 0.0,
        priority: 0,
        burst: 0,
        issuer: "".to_string(),
        status: RegionalCanaryAllocationStatus::Requested,
    }
}
