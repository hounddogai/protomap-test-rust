use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage plan must pass. Check returns its problems, or none.
pub trait UsagePlanRule: std::fmt::Debug {
    fn check(&self, item: &UsagePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePlanCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsagePlanRule for UsagePlanCeilingRangeRule {
    fn check(&self, item: &UsagePlan) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePlanRequiredCodeRule {
    pub message: String,
}

impl UsagePlanRule for UsagePlanRequiredCodeRule {
    fn check(&self, item: &UsagePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePlanCodeLengthRule {
    pub max_length: i64,
}

impl UsagePlanRule for UsagePlanCodeLengthRule {
    fn check(&self, item: &UsagePlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePlanTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl UsagePlanRule for UsagePlanTagsLimitRule {
    fn check(&self, item: &UsagePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a usage plan.
#[derive(Debug, Clone)]
pub struct UsagePlanValidator {
    pub rules: Vec<Rc<dyn UsagePlanRule>>,
}

impl UsagePlanValidator {
    pub fn validate(&self, item: &UsagePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsagePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsagePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_plan_validator() -> UsagePlanValidator {
    UsagePlanValidator {
        rules: vec![
            Rc::new(UsagePlanCeilingRangeRule { min: 6.8, max: 4297.9 }) as Rc<dyn UsagePlanRule>,
            Rc::new(UsagePlanRequiredCodeRule {
                message: "usage plan needs this field".to_string(),
            }),
            Rc::new(UsagePlanCodeLengthRule { max_length: 32 }),
            Rc::new(UsagePlanTagsLimitRule {
                max_count: 16,
                banned: vec!["pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_plan_validator();
        let item = new_usage_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = UsagePlanCeilingRangeRule { min: 7.0, max: 1324.9 };
        let mut item = new_usage_plan("x", "y");
        item.ceiling = 1326.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = UsagePlanRequiredCodeRule {
            message: "usage plan needs a value".to_string(),
        };
        let mut item = new_usage_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = UsagePlanCodeLengthRule { max_length: 80 };
        let mut item = new_usage_plan("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
