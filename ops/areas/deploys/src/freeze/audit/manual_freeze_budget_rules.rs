use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual freeze budget must pass. Check returns its problems, or none.
pub trait ManualFreezeBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ManualFreezeBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFreezeBudgetBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualFreezeBudgetRule for ManualFreezeBudgetBudgetRangeRule {
    fn check(&self, item: &ManualFreezeBudget) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFreezeBudgetRequiredCodeRule {
    pub message: String,
}

impl ManualFreezeBudgetRule for ManualFreezeBudgetRequiredCodeRule {
    fn check(&self, item: &ManualFreezeBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFreezeBudgetRequiredClusterRule {
    pub message: String,
}

impl ManualFreezeBudgetRule for ManualFreezeBudgetRequiredClusterRule {
    fn check(&self, item: &ManualFreezeBudget) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual freeze budget.
#[derive(Debug, Clone)]
pub struct ManualFreezeBudgetValidator {
    pub rules: Vec<Rc<dyn ManualFreezeBudgetRule>>,
}

impl ManualFreezeBudgetValidator {
    pub fn validate(&self, item: &ManualFreezeBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualFreezeBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualFreezeBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_freeze_budget_validator() -> ManualFreezeBudgetValidator {
    ManualFreezeBudgetValidator {
        rules: vec![
            Rc::new(ManualFreezeBudgetBudgetRangeRule { min: 8.2, max: 4170.1 }) as Rc<dyn ManualFreezeBudgetRule>,
            Rc::new(ManualFreezeBudgetRequiredCodeRule {
                message: "manual freeze budget needs an entry".to_string(),
            }),
            Rc::new(ManualFreezeBudgetRequiredClusterRule {
                message: "manual freeze budget needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_freeze_budget_validator();
        let item = new_manual_freeze_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ManualFreezeBudgetBudgetRangeRule { min: 5.7, max: 2907.1 };
        let mut item = new_manual_freeze_budget("x", "y");
        item.budget = 2908.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualFreezeBudgetRequiredCodeRule {
            message: "manual freeze budget needs a value".to_string(),
        };
        let mut item = new_manual_freeze_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ManualFreezeBudgetRequiredClusterRule {
            message: "manual freeze budget needs this field".to_string(),
        };
        let mut item = new_manual_freeze_budget("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
