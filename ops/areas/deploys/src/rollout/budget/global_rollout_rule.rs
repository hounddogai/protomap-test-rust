use super::*;

use acme_kit::text;

/// Where a global rollout rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRolloutRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalRolloutRuleStatus {
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

impl std::fmt::Display for GlobalRolloutRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global rollout rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutRule {
    pub code: String,
    pub label: String,
    pub host: String,
    pub rotation_days: i64,
    pub locale: String,
    pub score: f64,
    pub tenant_id: String,
    pub status: GlobalRolloutRuleStatus,
}

impl GlobalRolloutRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRolloutRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRolloutRuleStatus) -> bool {
        match self.status {
            GlobalRolloutRuleStatus::Idle => {
                return next == GlobalRolloutRuleStatus::Warming || next == GlobalRolloutRuleStatus::Stopped;
            }
            GlobalRolloutRuleStatus::Warming => {
                return next == GlobalRolloutRuleStatus::Draining;
            }
            GlobalRolloutRuleStatus::Ready => {
                return next == GlobalRolloutRuleStatus::Draining;
            }
            GlobalRolloutRuleStatus::Draining => {
                match next {
                    GlobalRolloutRuleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalRolloutRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global rollout rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.rotation_days as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRolloutRule {
        GlobalRolloutRule {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            rotation_days: self.rotation_days,
            locale: self.locale.clone(),
            score: self.score,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a global rollout rule in status idle.
pub fn new_global_rollout_rule(code: &str, label: &str) -> GlobalRolloutRule {
    GlobalRolloutRule {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        rotation_days: 0,
        locale: "".to_string(),
        score: 0.0,
        tenant_id: "".to_string(),
        status: GlobalRolloutRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_rollout_rule("code-312", "label-270");
        assert_eq!(item.status, GlobalRolloutRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_global_rollout_rule("code-312", "label-270");
        if item.move_to(GlobalRolloutRuleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRolloutRuleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_rollout_rule("code-312", "label-270");
        item.status = GlobalRolloutRuleStatus::Stopped;
        assert!(
            item.move_to(GlobalRolloutRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
