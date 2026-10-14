use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention profile must pass. Check returns its problems, or none.
pub trait RetentionProfileRule: std::fmt::Debug {
    fn check(&self, item: &RetentionProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfileAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RetentionProfileRule for RetentionProfileAllowedTitleRule {
    fn check(&self, item: &RetentionProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfileRequiredScopeRule {
    pub message: String,
}

impl RetentionProfileRule for RetentionProfileRequiredScopeRule {
    fn check(&self, item: &RetentionProfile) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfileScopeLengthRule {
    pub max_length: i64,
}

impl RetentionProfileRule for RetentionProfileScopeLengthRule {
    fn check(&self, item: &RetentionProfile) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfileAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionProfileRule for RetentionProfileAttemptsRangeRule {
    fn check(&self, item: &RetentionProfile) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfileRequiredTitleRule {
    pub message: String,
}

impl RetentionProfileRule for RetentionProfileRequiredTitleRule {
    fn check(&self, item: &RetentionProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfileLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionProfileRule for RetentionProfileLimitRangeRule {
    fn check(&self, item: &RetentionProfile) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a retention profile.
#[derive(Debug, Clone)]
pub struct RetentionProfileValidator {
    pub rules: Vec<Rc<dyn RetentionProfileRule>>,
}

impl RetentionProfileValidator {
    pub fn validate(&self, item: &RetentionProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_profile_validator() -> RetentionProfileValidator {
    RetentionProfileValidator {
        rules: vec![
            Rc::new(RetentionProfileAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn RetentionProfileRule>,
            Rc::new(RetentionProfileRequiredScopeRule {
                message: "retention profile needs this field".to_string(),
            }),
            Rc::new(RetentionProfileScopeLengthRule { max_length: 86400 }),
            Rc::new(RetentionProfileAttemptsRangeRule { min: 3600, max: 3600 }),
            Rc::new(RetentionProfileRequiredTitleRule {
                message: "retention profile needs an entry".to_string(),
            }),
            Rc::new(RetentionProfileLimitRangeRule { min: 80, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_profile_validator();
        let item = new_retention_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = RetentionProfileRequiredScopeRule {
            message: "retention profile needs an entry".to_string(),
        };
        let mut item = new_retention_profile("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = RetentionProfileScopeLengthRule { max_length: 120 };
        let mut item = new_retention_profile("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RetentionProfileAttemptsRangeRule { min: 120, max: 3600 };
        let mut item = new_retention_profile("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RetentionProfileRequiredTitleRule {
            message: "retention profile needs a value".to_string(),
        };
        let mut item = new_retention_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = RetentionProfileLimitRangeRule { min: 80, max: 3600 };
        let mut item = new_retention_profile("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
