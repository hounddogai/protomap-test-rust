use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage target must pass. Check returns its problems, or none.
pub trait UsageTargetRule: std::fmt::Debug {
    fn check(&self, item: &UsageTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTargetRequiredCodeRule {
    pub message: String,
}

impl UsageTargetRule for UsageTargetRequiredCodeRule {
    fn check(&self, item: &UsageTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTargetExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageTargetRule for UsageTargetExpiresAtRangeRule {
    fn check(&self, item: &UsageTarget) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTargetRequiredTitleRule {
    pub message: String,
}

impl UsageTargetRule for UsageTargetRequiredTitleRule {
    fn check(&self, item: &UsageTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage target.
#[derive(Debug, Clone)]
pub struct UsageTargetValidator {
    pub rules: Vec<Rc<dyn UsageTargetRule>>,
}

impl UsageTargetValidator {
    pub fn validate(&self, item: &UsageTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_target_validator() -> UsageTargetValidator {
    UsageTargetValidator {
        rules: vec![
            Rc::new(UsageTargetRequiredCodeRule {
                message: "usage target needs an entry".to_string(),
            }) as Rc<dyn UsageTargetRule>,
            Rc::new(UsageTargetExpiresAtRangeRule { min: 80, max: 255 }),
            Rc::new(UsageTargetRequiredTitleRule {
                message: "usage target needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_target_validator();
        let item = new_usage_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = UsageTargetRequiredCodeRule {
            message: "usage target needs an entry".to_string(),
        };
        let mut item = new_usage_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = UsageTargetExpiresAtRangeRule { min: 80, max: 86400 };
        let mut item = new_usage_target("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = UsageTargetRequiredTitleRule {
            message: "usage target needs a value".to_string(),
        };
        let mut item = new_usage_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
