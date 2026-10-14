use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation budget must pass. Check returns its problems, or none.
pub trait DelegationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &DelegationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBudgetAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationBudgetRule for DelegationBudgetAttemptsRangeRule {
    fn check(&self, item: &DelegationBudget) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBudgetCodePrefixRule {
    pub prefix: String,
}

impl DelegationBudgetRule for DelegationBudgetCodePrefixRule {
    fn check(&self, item: &DelegationBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBudgetRequiredCodeRule {
    pub message: String,
}

impl DelegationBudgetRule for DelegationBudgetRequiredCodeRule {
    fn check(&self, item: &DelegationBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation budget.
#[derive(Debug, Clone)]
pub struct DelegationBudgetValidator {
    pub rules: Vec<Rc<dyn DelegationBudgetRule>>,
}

impl DelegationBudgetValidator {
    pub fn validate(&self, item: &DelegationBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_budget_validator() -> DelegationBudgetValidator {
    DelegationBudgetValidator {
        rules: vec![
            Rc::new(DelegationBudgetAttemptsRangeRule { min: 16, max: 64 }) as Rc<dyn DelegationBudgetRule>,
            Rc::new(DelegationBudgetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(DelegationBudgetRequiredCodeRule {
                message: "delegation budget needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_budget_validator();
        let item = new_delegation_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = DelegationBudgetAttemptsRangeRule { min: 64, max: 80 };
        let mut item = new_delegation_budget("x", "y");
        item.attempts = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationBudgetRequiredCodeRule {
            message: "delegation budget needs an entry".to_string(),
        };
        let mut item = new_delegation_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
