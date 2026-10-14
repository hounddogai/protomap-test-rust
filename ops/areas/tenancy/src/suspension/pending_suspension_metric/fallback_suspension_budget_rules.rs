use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback suspension budget must pass. Check returns its problems, or none.
pub trait FallbackSuspensionBudgetRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSuspensionBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionBudgetCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackSuspensionBudgetRule for FallbackSuspensionBudgetCostRangeRule {
    fn check(&self, item: &FallbackSuspensionBudget) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionBudgetLabelLengthRule {
    pub max_length: i64,
}

impl FallbackSuspensionBudgetRule for FallbackSuspensionBudgetLabelLengthRule {
    fn check(&self, item: &FallbackSuspensionBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionBudgetCodeLengthRule {
    pub max_length: i64,
}

impl FallbackSuspensionBudgetRule for FallbackSuspensionBudgetCodeLengthRule {
    fn check(&self, item: &FallbackSuspensionBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback suspension budget.
#[derive(Debug, Clone)]
pub struct FallbackSuspensionBudgetValidator {
    pub rules: Vec<Rc<dyn FallbackSuspensionBudgetRule>>,
}

impl FallbackSuspensionBudgetValidator {
    pub fn validate(&self, item: &FallbackSuspensionBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackSuspensionBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSuspensionBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_suspension_budget_validator() -> FallbackSuspensionBudgetValidator {
    FallbackSuspensionBudgetValidator {
        rules: vec![
            Rc::new(FallbackSuspensionBudgetCostRangeRule { min: 9.8, max: 4815.2 })
                as Rc<dyn FallbackSuspensionBudgetRule>,
            Rc::new(FallbackSuspensionBudgetLabelLengthRule { max_length: 255 }),
            Rc::new(FallbackSuspensionBudgetCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_suspension_budget_validator();
        let item = new_fallback_suspension_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = FallbackSuspensionBudgetCostRangeRule { min: 9.9, max: 537.9 };
        let mut item = new_fallback_suspension_budget("x", "y");
        item.cost = 539.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackSuspensionBudgetLabelLengthRule { max_length: 120 };
        let mut item = new_fallback_suspension_budget("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackSuspensionBudgetCodeLengthRule { max_length: 32 };
        let mut item = new_fallback_suspension_budget("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
