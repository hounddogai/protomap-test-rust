use super::*;

use acme_kit::text;

/// Where a failover rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FailoverRuleStatus {
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

impl std::fmt::Display for FailoverRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRule {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub burst: i64,
    pub cluster: String,
    pub status: FailoverRuleStatus,
}

impl FailoverRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverRuleStatus) -> bool {
        match self.status {
            FailoverRuleStatus::Idle => {
                return next == FailoverRuleStatus::Ready || next == FailoverRuleStatus::Draining;
            }
            FailoverRuleStatus::Warming => {
                return next == FailoverRuleStatus::Ready;
            }
            FailoverRuleStatus::Ready => {
                return next == FailoverRuleStatus::Draining || next == FailoverRuleStatus::Stopped;
            }
            FailoverRuleStatus::Draining => {
                return next == FailoverRuleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> FailoverRule {
        FailoverRule {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            burst: self.burst,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a failover rule in status idle.
pub fn new_failover_rule(code: &str, name: &str) -> FailoverRule {
    FailoverRule {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        burst: 0,
        cluster: "".to_string(),
        status: FailoverRuleStatus::Idle,
    }
}
