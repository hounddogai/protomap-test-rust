use super::*;

use acme_kit::text;

/// Where a policy plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PolicyPlanStatus {
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

impl std::fmt::Display for PolicyPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyPlan {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub timezone: String,
    pub ends_at: i64,
    pub status: PolicyPlanStatus,
}

impl PolicyPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyPlanStatus) -> bool {
        match self.status {
            PolicyPlanStatus::New => {
                return next == PolicyPlanStatus::Done || next == PolicyPlanStatus::Failed;
            }
            PolicyPlanStatus::Queued => {
                match next {
                    PolicyPlanStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyPlanStatus::Running => {
                match next {
                    PolicyPlanStatus::Done | PolicyPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyPlanStatus::Done => {
                return next == PolicyPlanStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy plan", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ends_at as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyPlan {
        PolicyPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            timezone: self.timezone.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a policy plan in status new.
pub fn new_policy_plan(code: &str, label: &str) -> PolicyPlan {
    PolicyPlan {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        timezone: "".to_string(),
        ends_at: 0,
        status: PolicyPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_policy_plan("code-217", "label-200");
        assert_eq!(item.status, PolicyPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_policy_plan("code-217", "label-200");
        if item.move_to(PolicyPlanStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyPlanStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_policy_plan("code-217", "label-200");
        item.status = PolicyPlanStatus::Failed;
        assert!(item.move_to(PolicyPlanStatus::New).is_err(), "expected move to fail");
    }
}
