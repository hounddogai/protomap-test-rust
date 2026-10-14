use super::*;

use acme_kit::text;

/// Where a plan policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PlanPolicyStatus {
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

impl std::fmt::Display for PlanPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanPolicy {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub active: bool,
    pub phone: String,
    pub principal: String,
    pub status: PlanPolicyStatus,
}

impl PlanPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanPolicyStatus) -> bool {
        match self.status {
            PlanPolicyStatus::Idle => {
                return next == PlanPolicyStatus::Draining;
            }
            PlanPolicyStatus::Warming => {
                return next == PlanPolicyStatus::Ready || next == PlanPolicyStatus::Stopped;
            }
            PlanPolicyStatus::Ready => {
                return next == PlanPolicyStatus::Draining || next == PlanPolicyStatus::Stopped;
            }
            PlanPolicyStatus::Draining => {
                match next {
                    PlanPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PlanPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan policy", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> PlanPolicy {
        PlanPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            active: self.active,
            phone: self.phone.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a plan policy in status idle.
pub fn new_plan_policy(code: &str, name: &str) -> PlanPolicy {
    PlanPolicy {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        active: false,
        phone: "".to_string(),
        principal: "".to_string(),
        status: PlanPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_plan_policy("code-951", "name-360");
        assert_eq!(item.status, PlanPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_plan_policy("code-951", "name-360");
        if item.move_to(PlanPolicyStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanPolicyStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_plan_policy("code-951", "name-360");
        item.status = PlanPolicyStatus::Stopped;
        assert!(item.move_to(PlanPolicyStatus::Idle).is_err(), "expected move to fail");
    }
}
