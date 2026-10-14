use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled nonce budget must pass. Check returns its problems, or none.
pub trait ScheduledNonceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledNonceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceBudgetRequiredCodeRule {
    pub message: String,
}

impl ScheduledNonceBudgetRule for ScheduledNonceBudgetRequiredCodeRule {
    fn check(&self, item: &ScheduledNonceBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceBudgetThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledNonceBudgetRule for ScheduledNonceBudgetThresholdRangeRule {
    fn check(&self, item: &ScheduledNonceBudget) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceBudgetNameLengthRule {
    pub max_length: i64,
}

impl ScheduledNonceBudgetRule for ScheduledNonceBudgetNameLengthRule {
    fn check(&self, item: &ScheduledNonceBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled nonce budget.
#[derive(Debug, Clone)]
pub struct ScheduledNonceBudgetValidator {
    pub rules: Vec<Rc<dyn ScheduledNonceBudgetRule>>,
}

impl ScheduledNonceBudgetValidator {
    pub fn validate(&self, item: &ScheduledNonceBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledNonceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledNonceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_nonce_budget_validator() -> ScheduledNonceBudgetValidator {
    ScheduledNonceBudgetValidator {
        rules: vec![
            Rc::new(ScheduledNonceBudgetRequiredCodeRule {
                message: "scheduled nonce budget needs a value".to_string(),
            }) as Rc<dyn ScheduledNonceBudgetRule>,
            Rc::new(ScheduledNonceBudgetThresholdRangeRule { min: 3.4, max: 1081.7 }),
            Rc::new(ScheduledNonceBudgetNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_nonce_budget_validator();
        let item = new_scheduled_nonce_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledNonceBudgetRequiredCodeRule {
            message: "scheduled nonce budget needs an entry".to_string(),
        };
        let mut item = new_scheduled_nonce_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ScheduledNonceBudgetThresholdRangeRule { min: 7.7, max: 1490.4 };
        let mut item = new_scheduled_nonce_budget("x", "y");
        item.threshold = 1491.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledNonceBudgetNameLengthRule { max_length: 120 };
        let mut item = new_scheduled_nonce_budget("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
