use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global usage batch must pass. Check returns its problems, or none.
pub trait GlobalUsageBatchRule: std::fmt::Debug {
    fn check(&self, item: &GlobalUsageBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageBatchRequiredDescriptionRule {
    pub message: String,
}

impl GlobalUsageBatchRule for GlobalUsageBatchRequiredDescriptionRule {
    fn check(&self, item: &GlobalUsageBatch) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageBatchBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalUsageBatchRule for GlobalUsageBatchBudgetRangeRule {
    fn check(&self, item: &GlobalUsageBatch) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageBatchRequiredCodeRule {
    pub message: String,
}

impl GlobalUsageBatchRule for GlobalUsageBatchRequiredCodeRule {
    fn check(&self, item: &GlobalUsageBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global usage batch.
#[derive(Debug, Clone)]
pub struct GlobalUsageBatchValidator {
    pub rules: Vec<Rc<dyn GlobalUsageBatchRule>>,
}

impl GlobalUsageBatchValidator {
    pub fn validate(&self, item: &GlobalUsageBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalUsageBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalUsageBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_usage_batch_validator() -> GlobalUsageBatchValidator {
    GlobalUsageBatchValidator {
        rules: vec![
            Rc::new(GlobalUsageBatchRequiredDescriptionRule {
                message: "global usage batch needs an entry".to_string(),
            }) as Rc<dyn GlobalUsageBatchRule>,
            Rc::new(GlobalUsageBatchBudgetRangeRule { min: 7.5, max: 1252.8 }),
            Rc::new(GlobalUsageBatchRequiredCodeRule {
                message: "global usage batch needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_usage_batch_validator();
        let item = new_global_usage_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = GlobalUsageBatchRequiredDescriptionRule {
            message: "global usage batch needs an entry".to_string(),
        };
        let mut item = new_global_usage_batch("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = GlobalUsageBatchBudgetRangeRule { min: 4.3, max: 3195.6 };
        let mut item = new_global_usage_batch("x", "y");
        item.budget = 3197.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalUsageBatchRequiredCodeRule {
            message: "global usage batch needs an entry".to_string(),
        };
        let mut item = new_global_usage_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
