use super::*;

use acme_kit::text;

/// Where a adaptive isolation rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveIsolationRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveIsolationRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AdaptiveIsolationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive isolation rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationRule {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub visible: bool,
    pub burst: i64,
    pub status: AdaptiveIsolationRuleStatus,
}

impl AdaptiveIsolationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveIsolationRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveIsolationRuleStatus) -> bool {
        match self.status {
            AdaptiveIsolationRuleStatus::New => {
                return next == AdaptiveIsolationRuleStatus::Done || next == AdaptiveIsolationRuleStatus::Failed;
            }
            AdaptiveIsolationRuleStatus::Queued => {
                match next {
                    AdaptiveIsolationRuleStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveIsolationRuleStatus::Running => {
                return next == AdaptiveIsolationRuleStatus::Done;
            }
            AdaptiveIsolationRuleStatus::Done => {
                match next {
                    AdaptiveIsolationRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveIsolationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive isolation rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive isolation rule", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> AdaptiveIsolationRule {
        AdaptiveIsolationRule {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            visible: self.visible,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive isolation rule in status new.
pub fn new_adaptive_isolation_rule(code: &str, name: &str) -> AdaptiveIsolationRule {
    AdaptiveIsolationRule {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        visible: false,
        burst: 0,
        status: AdaptiveIsolationRuleStatus::New,
    }
}
