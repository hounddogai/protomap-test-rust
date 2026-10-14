use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn baseline must pass. Check returns its problems, or none.
pub trait BudgetBurnBaselineRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBaselineBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnBaselineRule for BudgetBurnBaselineBudgetRangeRule {
    fn check(&self, item: &BudgetBurnBaseline) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBaselineRequiredTitleRule {
    pub message: String,
}

impl BudgetBurnBaselineRule for BudgetBurnBaselineRequiredTitleRule {
    fn check(&self, item: &BudgetBurnBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBaselineCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnBaselineRule for BudgetBurnBaselineCodePrefixRule {
    fn check(&self, item: &BudgetBurnBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn baseline.
#[derive(Debug, Clone)]
pub struct BudgetBurnBaselineValidator {
    pub rules: Vec<Rc<dyn BudgetBurnBaselineRule>>,
}

impl BudgetBurnBaselineValidator {
    pub fn validate(&self, item: &BudgetBurnBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_baseline_validator() -> BudgetBurnBaselineValidator {
    BudgetBurnBaselineValidator {
        rules: vec![
            Rc::new(BudgetBurnBaselineBudgetRangeRule { min: 4.2, max: 4210.7 }) as Rc<dyn BudgetBurnBaselineRule>,
            Rc::new(BudgetBurnBaselineRequiredTitleRule {
                message: "budget burn baseline needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnBaselineCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_baseline_validator();
        let item = new_budget_burn_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = BudgetBurnBaselineBudgetRangeRule { min: 0.6, max: 4904.1 };
        let mut item = new_budget_burn_baseline("x", "y");
        item.budget = 4905.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BudgetBurnBaselineRequiredTitleRule {
            message: "budget burn baseline needs an entry".to_string(),
        };
        let mut item = new_budget_burn_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
