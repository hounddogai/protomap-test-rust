use super::*;

use acme_kit::text;

/// Where a identity rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IdentityRuleStatus {
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

impl std::fmt::Display for IdentityRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity rule as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRule {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub revision: i64,
    pub priority: i64,
    pub status: IdentityRuleStatus,
}

impl IdentityRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityRuleStatus) -> bool {
        match self.status {
            IdentityRuleStatus::Idle => {
                return next == IdentityRuleStatus::Stopped;
            }
            IdentityRuleStatus::Warming => {
                match next {
                    IdentityRuleStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityRuleStatus::Ready => {
                match next {
                    IdentityRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityRuleStatus::Draining => {
                return next == IdentityRuleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.priority as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityRule {
        IdentityRule {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            revision: self.revision,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a identity rule in status idle.
pub fn new_identity_rule(code: &str, label: &str) -> IdentityRule {
    IdentityRule {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        revision: 0,
        priority: 0,
        status: IdentityRuleStatus::Idle,
    }
}
