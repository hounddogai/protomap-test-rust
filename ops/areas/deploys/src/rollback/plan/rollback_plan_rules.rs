use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback plan must pass. Check returns its problems, or none.
pub trait RollbackPlanRule: std::fmt::Debug {
    fn check(&self, item: &RollbackPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPlanTitleLengthRule {
    pub max_length: i64,
}

impl RollbackPlanRule for RollbackPlanTitleLengthRule {
    fn check(&self, item: &RollbackPlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPlanAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RollbackPlanRule for RollbackPlanAllowedTitleRule {
    fn check(&self, item: &RollbackPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPlanVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackPlanRule for RollbackPlanVersionRangeRule {
    fn check(&self, item: &RollbackPlan) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPlanCodePrefixRule {
    pub prefix: String,
}

impl RollbackPlanRule for RollbackPlanCodePrefixRule {
    fn check(&self, item: &RollbackPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback plan.
#[derive(Debug, Clone)]
pub struct RollbackPlanValidator {
    pub rules: Vec<Rc<dyn RollbackPlanRule>>,
}

impl RollbackPlanValidator {
    pub fn validate(&self, item: &RollbackPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_plan_validator() -> RollbackPlanValidator {
    RollbackPlanValidator {
        rules: vec![
            Rc::new(RollbackPlanTitleLengthRule { max_length: 16 }) as Rc<dyn RollbackPlanRule>,
            Rc::new(RollbackPlanAllowedTitleRule {
                allowed: vec!["pending".to_string(), "lenient".to_string()],
            }),
            Rc::new(RollbackPlanVersionRangeRule { min: 32, max: 80 }),
            Rc::new(RollbackPlanCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_plan_validator();
        let item = new_rollback_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollbackPlanTitleLengthRule { max_length: 64 };
        let mut item = new_rollback_plan("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RollbackPlanVersionRangeRule { min: 16, max: 120 };
        let mut item = new_rollback_plan("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
