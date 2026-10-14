use super::*;

use acme_kit::text;

/// Where a ttl policy plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TtlPolicyPlanStatus {
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

impl std::fmt::Display for TtlPolicyPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyPlan {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub grace_days: i64,
    pub segment: String,
    pub margin: f64,
    pub principal: String,
    pub rate: f64,
    pub archived: bool,
    pub priority: i64,
    pub cluster: String,
    pub status: TtlPolicyPlanStatus,
}

impl TtlPolicyPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyPlanStatus) -> bool {
        match self.status {
            TtlPolicyPlanStatus::Idle => {
                return next == TtlPolicyPlanStatus::Ready || next == TtlPolicyPlanStatus::Stopped;
            }
            TtlPolicyPlanStatus::Warming => {
                match next {
                    TtlPolicyPlanStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyPlanStatus::Ready => {
                return next == TtlPolicyPlanStatus::Stopped;
            }
            TtlPolicyPlanStatus::Draining => {
                match next {
                    TtlPolicyPlanStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TtlPolicyPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.rate * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyPlan {
        TtlPolicyPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            grace_days: self.grace_days,
            segment: self.segment.clone(),
            margin: self.margin,
            principal: self.principal.clone(),
            rate: self.rate,
            archived: self.archived,
            priority: self.priority,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a ttl policy plan in status idle.
pub fn new_ttl_policy_plan(code: &str, label: &str) -> TtlPolicyPlan {
    TtlPolicyPlan {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        grace_days: 0,
        segment: "".to_string(),
        margin: 0.0,
        principal: "".to_string(),
        rate: 0.0,
        archived: false,
        priority: 0,
        cluster: "".to_string(),
        status: TtlPolicyPlanStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_ttl_policy_plan("code-88", "label-794");
        assert_eq!(item.status, TtlPolicyPlanStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_ttl_policy_plan("code-88", "label-794");
        if item.move_to(TtlPolicyPlanStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyPlanStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_ttl_policy_plan("code-88", "label-794");
        item.status = TtlPolicyPlanStatus::Stopped;
        assert!(
            item.move_to(TtlPolicyPlanStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
