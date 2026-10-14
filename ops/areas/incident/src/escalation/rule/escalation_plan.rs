use super::*;

use acme_kit::text;

/// Where a escalation plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EscalationPlanStatus {
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

impl std::fmt::Display for EscalationPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One escalation plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPlan {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub approved: bool,
    pub summary: String,
    pub status: EscalationPlanStatus,
}

impl EscalationPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationPlanStatus) -> bool {
        match self.status {
            EscalationPlanStatus::Idle => {
                match next {
                    EscalationPlanStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationPlanStatus::Warming => {
                match next {
                    EscalationPlanStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationPlanStatus::Ready => {
                return next == EscalationPlanStatus::Draining;
            }
            EscalationPlanStatus::Draining => {
                return next == EscalationPlanStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> EscalationPlan {
        EscalationPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            approved: self.approved,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation plan in status idle.
pub fn new_escalation_plan(code: &str, name: &str) -> EscalationPlan {
    EscalationPlan {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        approved: false,
        summary: "".to_string(),
        status: EscalationPlanStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_escalation_plan("code-552", "name-598");
        assert_eq!(item.status, EscalationPlanStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_escalation_plan("code-552", "name-598");
        if item.move_to(EscalationPlanStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationPlanStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_escalation_plan("code-552", "name-598");
        item.status = EscalationPlanStatus::Stopped;
        assert!(
            item.move_to(EscalationPlanStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
