use super::*;

use acme_kit::text;

/// Where a plan rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PlanRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PlanRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRule {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub key_id: String,
    pub ceiling: f64,
    pub attempts: i64,
    pub status: PlanRuleStatus,
}

impl PlanRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanRuleStatus) -> bool {
        match self.status {
            PlanRuleStatus::Proposed => {
                return next == PlanRuleStatus::Approved || next == PlanRuleStatus::Withdrawn;
            }
            PlanRuleStatus::Approved => {
                return next == PlanRuleStatus::Withdrawn;
            }
            PlanRuleStatus::Rejected => {
                match next {
                    PlanRuleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PlanRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.burst as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanRule {
        PlanRule {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            key_id: self.key_id.clone(),
            ceiling: self.ceiling,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a plan rule in status proposed.
pub fn new_plan_rule(code: &str, title: &str) -> PlanRule {
    PlanRule {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        key_id: "".to_string(),
        ceiling: 0.0,
        attempts: 0,
        status: PlanRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_plan_rule("code-555", "title-434");
        assert_eq!(item.status, PlanRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_plan_rule("code-555", "title-434");
        if item.move_to(PlanRuleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanRuleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_plan_rule("code-555", "title-434");
        item.status = PlanRuleStatus::Withdrawn;
        assert!(item.move_to(PlanRuleStatus::Proposed).is_err(), "expected move to fail");
    }
}
