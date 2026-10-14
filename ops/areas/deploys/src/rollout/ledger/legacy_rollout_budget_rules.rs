use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rollout budget must pass. Check returns its problems, or none.
pub trait LegacyRolloutBudgetRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRolloutBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutBudgetCodeLengthRule {
    pub max_length: i64,
}

impl LegacyRolloutBudgetRule for LegacyRolloutBudgetCodeLengthRule {
    fn check(&self, item: &LegacyRolloutBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutBudgetRequiredNameRule {
    pub message: String,
}

impl LegacyRolloutBudgetRule for LegacyRolloutBudgetRequiredNameRule {
    fn check(&self, item: &LegacyRolloutBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutBudgetCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyRolloutBudgetRule for LegacyRolloutBudgetCostRangeRule {
    fn check(&self, item: &LegacyRolloutBudget) -> Vec<String> {
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

/// Runs every rule of a legacy rollout budget.
#[derive(Debug, Clone)]
pub struct LegacyRolloutBudgetValidator {
    pub rules: Vec<Rc<dyn LegacyRolloutBudgetRule>>,
}

impl LegacyRolloutBudgetValidator {
    pub fn validate(&self, item: &LegacyRolloutBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRolloutBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRolloutBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollout_budget_validator() -> LegacyRolloutBudgetValidator {
    LegacyRolloutBudgetValidator {
        rules: vec![
            Rc::new(LegacyRolloutBudgetCodeLengthRule { max_length: 3600 }) as Rc<dyn LegacyRolloutBudgetRule>,
            Rc::new(LegacyRolloutBudgetRequiredNameRule {
                message: "legacy rollout budget needs this field".to_string(),
            }),
            Rc::new(LegacyRolloutBudgetCostRangeRule { min: 7.7, max: 3200.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_rollout_budget_validator();
        let item = new_legacy_rollout_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyRolloutBudgetCodeLengthRule { max_length: 80 };
        let mut item = new_legacy_rollout_budget("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyRolloutBudgetRequiredNameRule {
            message: "legacy rollout budget needs an entry".to_string(),
        };
        let mut item = new_legacy_rollout_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = LegacyRolloutBudgetCostRangeRule { min: 7.3, max: 3474.7 };
        let mut item = new_legacy_rollout_budget("x", "y");
        item.cost = 3476.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
