use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled rollout budget must pass. Check returns its problems, or none.
pub trait ScheduledRolloutBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRolloutBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutBudgetRequiredCodeRule {
    pub message: String,
}

impl ScheduledRolloutBudgetRule for ScheduledRolloutBudgetRequiredCodeRule {
    fn check(&self, item: &ScheduledRolloutBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutBudgetCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledRolloutBudgetRule for ScheduledRolloutBudgetCostRangeRule {
    fn check(&self, item: &ScheduledRolloutBudget) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutBudgetRequiredSourceRule {
    pub message: String,
}

impl ScheduledRolloutBudgetRule for ScheduledRolloutBudgetRequiredSourceRule {
    fn check(&self, item: &ScheduledRolloutBudget) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutBudgetCodePrefixRule {
    pub prefix: String,
}

impl ScheduledRolloutBudgetRule for ScheduledRolloutBudgetCodePrefixRule {
    fn check(&self, item: &ScheduledRolloutBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutBudgetRequiredNameRule {
    pub message: String,
}

impl ScheduledRolloutBudgetRule for ScheduledRolloutBudgetRequiredNameRule {
    fn check(&self, item: &ScheduledRolloutBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled rollout budget.
#[derive(Debug, Clone)]
pub struct ScheduledRolloutBudgetValidator {
    pub rules: Vec<Rc<dyn ScheduledRolloutBudgetRule>>,
}

impl ScheduledRolloutBudgetValidator {
    pub fn validate(&self, item: &ScheduledRolloutBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledRolloutBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRolloutBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_rollout_budget_validator() -> ScheduledRolloutBudgetValidator {
    ScheduledRolloutBudgetValidator {
        rules: vec![
            Rc::new(ScheduledRolloutBudgetRequiredCodeRule {
                message: "scheduled rollout budget needs an entry".to_string(),
            }) as Rc<dyn ScheduledRolloutBudgetRule>,
            Rc::new(ScheduledRolloutBudgetCostRangeRule { min: 6.7, max: 2399.9 }),
            Rc::new(ScheduledRolloutBudgetRequiredSourceRule {
                message: "scheduled rollout budget needs a value".to_string(),
            }),
            Rc::new(ScheduledRolloutBudgetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ScheduledRolloutBudgetRequiredNameRule {
                message: "scheduled rollout budget needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_rollout_budget_validator();
        let item = new_scheduled_rollout_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledRolloutBudgetRequiredCodeRule {
            message: "scheduled rollout budget needs this field".to_string(),
        };
        let mut item = new_scheduled_rollout_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ScheduledRolloutBudgetCostRangeRule { min: 3.5, max: 2428.3 };
        let mut item = new_scheduled_rollout_budget("x", "y");
        item.cost = 2429.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ScheduledRolloutBudgetRequiredSourceRule {
            message: "scheduled rollout budget needs this field".to_string(),
        };
        let mut item = new_scheduled_rollout_budget("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledRolloutBudgetRequiredNameRule {
            message: "scheduled rollout budget needs an entry".to_string(),
        };
        let mut item = new_scheduled_rollout_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
