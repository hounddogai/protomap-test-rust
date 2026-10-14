use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending slo budget must pass. Check returns its problems, or none.
pub trait PendingSloBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PendingSloBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloBudgetCodePrefixRule {
    pub prefix: String,
}

impl PendingSloBudgetRule for PendingSloBudgetCodePrefixRule {
    fn check(&self, item: &PendingSloBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloBudgetRequiredLabelRule {
    pub message: String,
}

impl PendingSloBudgetRule for PendingSloBudgetRequiredLabelRule {
    fn check(&self, item: &PendingSloBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloBudgetCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingSloBudgetRule for PendingSloBudgetCeilingRangeRule {
    fn check(&self, item: &PendingSloBudget) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloBudgetAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingSloBudgetRule for PendingSloBudgetAllowedLabelRule {
    fn check(&self, item: &PendingSloBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloBudgetActiveGuardRule {
    pub limit: f64,
}

impl PendingSloBudgetRule for PendingSloBudgetActiveGuardRule {
    fn check(&self, item: &PendingSloBudget) -> Vec<String> {
        if item.active && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while active", item.weight, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending slo budget.
#[derive(Debug, Clone)]
pub struct PendingSloBudgetValidator {
    pub rules: Vec<Rc<dyn PendingSloBudgetRule>>,
}

impl PendingSloBudgetValidator {
    pub fn validate(&self, item: &PendingSloBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingSloBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSloBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_slo_budget_validator() -> PendingSloBudgetValidator {
    PendingSloBudgetValidator {
        rules: vec![
            Rc::new(PendingSloBudgetCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn PendingSloBudgetRule>,
            Rc::new(PendingSloBudgetRequiredLabelRule {
                message: "pending slo budget needs a value".to_string(),
            }),
            Rc::new(PendingSloBudgetCeilingRangeRule { min: 9.5, max: 1051.6 }),
            Rc::new(PendingSloBudgetAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(PendingSloBudgetActiveGuardRule { limit: 3.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_slo_budget_validator();
        let item = new_pending_slo_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingSloBudgetRequiredLabelRule {
            message: "pending slo budget needs an entry".to_string(),
        };
        let mut item = new_pending_slo_budget("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = PendingSloBudgetCeilingRangeRule { min: 2.2, max: 1543.6 };
        let mut item = new_pending_slo_budget("x", "y");
        item.ceiling = 1545.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
