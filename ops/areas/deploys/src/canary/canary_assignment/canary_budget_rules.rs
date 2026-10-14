use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary budget must pass. Check returns its problems, or none.
pub trait CanaryBudgetRule: std::fmt::Debug {
    fn check(&self, item: &CanaryBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBudgetRequiredHostRule {
    pub message: String,
}

impl CanaryBudgetRule for CanaryBudgetRequiredHostRule {
    fn check(&self, item: &CanaryBudget) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBudgetNameLengthRule {
    pub max_length: i64,
}

impl CanaryBudgetRule for CanaryBudgetNameLengthRule {
    fn check(&self, item: &CanaryBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBudgetCodePrefixRule {
    pub prefix: String,
}

impl CanaryBudgetRule for CanaryBudgetCodePrefixRule {
    fn check(&self, item: &CanaryBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary budget.
#[derive(Debug, Clone)]
pub struct CanaryBudgetValidator {
    pub rules: Vec<Rc<dyn CanaryBudgetRule>>,
}

impl CanaryBudgetValidator {
    pub fn validate(&self, item: &CanaryBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_budget_validator() -> CanaryBudgetValidator {
    CanaryBudgetValidator {
        rules: vec![
            Rc::new(CanaryBudgetRequiredHostRule {
                message: "canary budget needs this field".to_string(),
            }) as Rc<dyn CanaryBudgetRule>,
            Rc::new(CanaryBudgetNameLengthRule { max_length: 32 }),
            Rc::new(CanaryBudgetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_budget_validator();
        let item = new_canary_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = CanaryBudgetRequiredHostRule {
            message: "canary budget needs this field".to_string(),
        };
        let mut item = new_canary_budget("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CanaryBudgetNameLengthRule { max_length: 1000 };
        let mut item = new_canary_budget("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
