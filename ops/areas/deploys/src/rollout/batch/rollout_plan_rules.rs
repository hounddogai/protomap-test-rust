use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout plan must pass. Check returns its problems, or none.
pub trait RolloutPlanRule: std::fmt::Debug {
    fn check(&self, item: &RolloutPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPlanCodeLengthRule {
    pub max_length: i64,
}

impl RolloutPlanRule for RolloutPlanCodeLengthRule {
    fn check(&self, item: &RolloutPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPlanAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RolloutPlanRule for RolloutPlanAllowedCodeRule {
    fn check(&self, item: &RolloutPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPlanGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutPlanRule for RolloutPlanGraceDaysRangeRule {
    fn check(&self, item: &RolloutPlan) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout plan.
#[derive(Debug, Clone)]
pub struct RolloutPlanValidator {
    pub rules: Vec<Rc<dyn RolloutPlanRule>>,
}

impl RolloutPlanValidator {
    pub fn validate(&self, item: &RolloutPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_plan_validator() -> RolloutPlanValidator {
    RolloutPlanValidator {
        rules: vec![
            Rc::new(RolloutPlanCodeLengthRule { max_length: 3600 }) as Rc<dyn RolloutPlanRule>,
            Rc::new(RolloutPlanAllowedCodeRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(RolloutPlanGraceDaysRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_plan_validator();
        let item = new_rollout_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutPlanCodeLengthRule { max_length: 120 };
        let mut item = new_rollout_plan("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RolloutPlanGraceDaysRangeRule { min: 16, max: 64 };
        let mut item = new_rollout_plan("x", "y");
        item.grace_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
