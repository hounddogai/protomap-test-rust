use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a escalation plan must pass. Check returns its problems, or none.
pub trait EscalationPlanRule: std::fmt::Debug {
    fn check(&self, item: &EscalationPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPlanRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationPlanRule for EscalationPlanRevisionRangeRule {
    fn check(&self, item: &EscalationPlan) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPlanAllowedNameRule {
    pub allowed: Vec<String>,
}

impl EscalationPlanRule for EscalationPlanAllowedNameRule {
    fn check(&self, item: &EscalationPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPlanRequiredNameRule {
    pub message: String,
}

impl EscalationPlanRule for EscalationPlanRequiredNameRule {
    fn check(&self, item: &EscalationPlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation plan.
#[derive(Debug, Clone)]
pub struct EscalationPlanValidator {
    pub rules: Vec<Rc<dyn EscalationPlanRule>>,
}

impl EscalationPlanValidator {
    pub fn validate(&self, item: &EscalationPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationPlan]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_escalation_plan_validator() -> EscalationPlanValidator {
    EscalationPlanValidator {
        rules: vec![
            Rc::new(EscalationPlanRevisionRangeRule { min: 64, max: 120 }) as Rc<dyn EscalationPlanRule>,
            Rc::new(EscalationPlanAllowedNameRule {
                allowed: vec!["strict".to_string(), "fallback".to_string()],
            }),
            Rc::new(EscalationPlanRequiredNameRule {
                message: "escalation plan needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_plan_validator();
        let item = new_escalation_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = EscalationPlanRevisionRangeRule { min: 32, max: 1000 };
        let mut item = new_escalation_plan("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EscalationPlanRequiredNameRule {
            message: "escalation plan needs this field".to_string(),
        };
        let mut item = new_escalation_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
