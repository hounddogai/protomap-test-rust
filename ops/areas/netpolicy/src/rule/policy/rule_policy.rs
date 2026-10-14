use super::*;

use acme_kit::text;

/// Where a rule policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RulePolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RulePolicyStatus {
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

impl std::fmt::Display for RulePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RulePolicy {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub notes: String,
    pub version: i64,
    pub window_days: i64,
    pub description: String,
    pub rate: f64,
    pub status: RulePolicyStatus,
}

impl RulePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RulePolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RulePolicyStatus) -> bool {
        match self.status {
            RulePolicyStatus::Idle => {
                return next == RulePolicyStatus::Ready;
            }
            RulePolicyStatus::Warming => {
                return next == RulePolicyStatus::Ready || next == RulePolicyStatus::Draining;
            }
            RulePolicyStatus::Ready => {
                return next == RulePolicyStatus::Stopped;
            }
            RulePolicyStatus::Draining => {
                match next {
                    RulePolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RulePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.window_days as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RulePolicy {
        RulePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            notes: self.notes.clone(),
            version: self.version,
            window_days: self.window_days,
            description: self.description.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rule policy in status idle.
pub fn new_rule_policy(code: &str, name: &str) -> RulePolicy {
    RulePolicy {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        notes: "".to_string(),
        version: 0,
        window_days: 0,
        description: "".to_string(),
        rate: 0.0,
        status: RulePolicyStatus::Idle,
    }
}
