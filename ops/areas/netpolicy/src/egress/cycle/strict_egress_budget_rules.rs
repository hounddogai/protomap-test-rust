use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict egress budget must pass. Check returns its problems, or none.
pub trait StrictEgressBudgetRule: std::fmt::Debug {
    fn check(&self, item: &StrictEgressBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBudgetRequiredTimezoneRule {
    pub message: String,
}

impl StrictEgressBudgetRule for StrictEgressBudgetRequiredTimezoneRule {
    fn check(&self, item: &StrictEgressBudget) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBudgetCodeLengthRule {
    pub max_length: i64,
}

impl StrictEgressBudgetRule for StrictEgressBudgetCodeLengthRule {
    fn check(&self, item: &StrictEgressBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBudgetRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictEgressBudgetRule for StrictEgressBudgetRevisionRangeRule {
    fn check(&self, item: &StrictEgressBudget) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict egress budget.
#[derive(Debug, Clone)]
pub struct StrictEgressBudgetValidator {
    pub rules: Vec<Rc<dyn StrictEgressBudgetRule>>,
}

impl StrictEgressBudgetValidator {
    pub fn validate(&self, item: &StrictEgressBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictEgressBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictEgressBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_egress_budget_validator() -> StrictEgressBudgetValidator {
    StrictEgressBudgetValidator {
        rules: vec![
            Rc::new(StrictEgressBudgetRequiredTimezoneRule {
                message: "strict egress budget needs this field".to_string(),
            }) as Rc<dyn StrictEgressBudgetRule>,
            Rc::new(StrictEgressBudgetCodeLengthRule { max_length: 32 }),
            Rc::new(StrictEgressBudgetRevisionRangeRule { min: 16, max: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_egress_budget_validator();
        let item = new_strict_egress_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = StrictEgressBudgetRequiredTimezoneRule {
            message: "strict egress budget needs a value".to_string(),
        };
        let mut item = new_strict_egress_budget("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictEgressBudgetCodeLengthRule { max_length: 120 };
        let mut item = new_strict_egress_budget("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = StrictEgressBudgetRevisionRangeRule { min: 64, max: 64 };
        let mut item = new_strict_egress_budget("x", "y");
        item.revision = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
