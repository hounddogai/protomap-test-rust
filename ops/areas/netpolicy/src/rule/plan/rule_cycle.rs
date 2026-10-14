use super::*;

use acme_kit::text;

/// Where a rule cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RuleCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RuleCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule cycle as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleCycle {
    pub code: String,
    pub name: String,
    pub label: String,
    pub currency: String,
    pub external_ref: String,
    pub budget: f64,
    pub duration_minutes: i64,
    pub approved: bool,
    pub principal: String,
    pub namespace: String,
    pub status: RuleCycleStatus,
}

impl RuleCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleCycleStatus) -> bool {
        match self.status {
            RuleCycleStatus::Proposed => {
                match next {
                    RuleCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleCycleStatus::Approved => {
                match next {
                    RuleCycleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleCycleStatus::Rejected => {
                match next {
                    RuleCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RuleCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.duration_minutes as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleCycle {
        RuleCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            external_ref: self.external_ref.clone(),
            budget: self.budget,
            duration_minutes: self.duration_minutes,
            approved: self.approved,
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rule cycle in status proposed.
pub fn new_rule_cycle(code: &str, name: &str) -> RuleCycle {
    RuleCycle {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        currency: "".to_string(),
        external_ref: "".to_string(),
        budget: 0.0,
        duration_minutes: 0,
        approved: false,
        principal: "".to_string(),
        namespace: "".to_string(),
        status: RuleCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rule_cycle("code-611", "name-576");
        assert_eq!(item.status, RuleCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_rule_cycle("code-611", "name-576");
        if item.move_to(RuleCycleStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleCycleStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rule_cycle("code-611", "name-576");
        item.status = RuleCycleStatus::Withdrawn;
        assert!(
            item.move_to(RuleCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
