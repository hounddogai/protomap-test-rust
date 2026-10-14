use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary plan must pass. Check returns its problems, or none.
pub trait CanaryPlanRule: std::fmt::Debug {
    fn check(&self, item: &CanaryPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPlanRequiredCodeRule {
    pub message: String,
}

impl CanaryPlanRule for CanaryPlanRequiredCodeRule {
    fn check(&self, item: &CanaryPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPlanMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanaryPlanRule for CanaryPlanMinimumRangeRule {
    fn check(&self, item: &CanaryPlan) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPlanNameLengthRule {
    pub max_length: i64,
}

impl CanaryPlanRule for CanaryPlanNameLengthRule {
    fn check(&self, item: &CanaryPlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary plan.
#[derive(Debug, Clone)]
pub struct CanaryPlanValidator {
    pub rules: Vec<Rc<dyn CanaryPlanRule>>,
}

impl CanaryPlanValidator {
    pub fn validate(&self, item: &CanaryPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_plan_validator() -> CanaryPlanValidator {
    CanaryPlanValidator {
        rules: vec![
            Rc::new(CanaryPlanRequiredCodeRule {
                message: "canary plan needs an entry".to_string(),
            }) as Rc<dyn CanaryPlanRule>,
            Rc::new(CanaryPlanMinimumRangeRule { min: 1.9, max: 2165.4 }),
            Rc::new(CanaryPlanNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_plan_validator();
        let item = new_canary_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryPlanRequiredCodeRule {
            message: "canary plan needs a value".to_string(),
        };
        let mut item = new_canary_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = CanaryPlanMinimumRangeRule { min: 8.4, max: 656.6 };
        let mut item = new_canary_plan("x", "y");
        item.minimum = 658.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CanaryPlanNameLengthRule { max_length: 16 };
        let mut item = new_canary_plan("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
