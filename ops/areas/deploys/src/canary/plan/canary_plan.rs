use super::*;

use acme_kit::text;

/// Where a canary plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryPlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CanaryPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CanaryPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPlan {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub minimum: f64,
    pub title: String,
    pub status: CanaryPlanStatus,
}

impl CanaryPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryPlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryPlanStatus) -> bool {
        match self.status {
            CanaryPlanStatus::Planned => {
                match next {
                    CanaryPlanStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryPlanStatus::Scheduled => {
                return next == CanaryPlanStatus::InProgress || next == CanaryPlanStatus::Cancelled;
            }
            CanaryPlanStatus::InProgress => {
                match next {
                    CanaryPlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryPlanStatus::Completed => {
                match next {
                    CanaryPlanStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CanaryPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "canary plan", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> CanaryPlan {
        CanaryPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            minimum: self.minimum,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a canary plan in status planned.
pub fn new_canary_plan(code: &str, name: &str) -> CanaryPlan {
    CanaryPlan {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        minimum: 0.0,
        title: "".to_string(),
        status: CanaryPlanStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_canary_plan("code-439", "name-919");
        assert_eq!(item.status, CanaryPlanStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_canary_plan("code-439", "name-919");
        if item.move_to(CanaryPlanStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryPlanStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_canary_plan("code-439", "name-919");
        item.status = CanaryPlanStatus::Cancelled;
        assert!(
            item.move_to(CanaryPlanStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
