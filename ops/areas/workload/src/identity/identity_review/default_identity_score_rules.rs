use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default identity score must pass. Check returns its problems, or none.
pub trait DefaultIdentityScoreRule: std::fmt::Debug {
    fn check(&self, item: &DefaultIdentityScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityScoreWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultIdentityScoreRule for DefaultIdentityScoreWindowDaysRangeRule {
    fn check(&self, item: &DefaultIdentityScore) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityScoreAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultIdentityScoreRule for DefaultIdentityScoreAttemptsRangeRule {
    fn check(&self, item: &DefaultIdentityScore) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityScoreCodeLengthRule {
    pub max_length: i64,
}

impl DefaultIdentityScoreRule for DefaultIdentityScoreCodeLengthRule {
    fn check(&self, item: &DefaultIdentityScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityScorePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultIdentityScoreRule for DefaultIdentityScorePriorityRangeRule {
    fn check(&self, item: &DefaultIdentityScore) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultIdentityScoreRule for DefaultIdentityScoreAllowedCodeRule {
    fn check(&self, item: &DefaultIdentityScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default identity score.
#[derive(Debug, Clone)]
pub struct DefaultIdentityScoreValidator {
    pub rules: Vec<Rc<dyn DefaultIdentityScoreRule>>,
}

impl DefaultIdentityScoreValidator {
    pub fn validate(&self, item: &DefaultIdentityScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultIdentityScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultIdentityScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_identity_score_validator() -> DefaultIdentityScoreValidator {
    DefaultIdentityScoreValidator {
        rules: vec![
            Rc::new(DefaultIdentityScoreWindowDaysRangeRule { min: 32, max: 1000 }) as Rc<dyn DefaultIdentityScoreRule>,
            Rc::new(DefaultIdentityScoreAttemptsRangeRule { min: 16, max: 1000 }),
            Rc::new(DefaultIdentityScoreCodeLengthRule { max_length: 1000 }),
            Rc::new(DefaultIdentityScorePriorityRangeRule { min: 16, max: 86400 }),
            Rc::new(DefaultIdentityScoreAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_identity_score_validator();
        let item = new_default_identity_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = DefaultIdentityScoreWindowDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_default_identity_score("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = DefaultIdentityScoreAttemptsRangeRule { min: 255, max: 3600 };
        let mut item = new_default_identity_score("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultIdentityScoreCodeLengthRule { max_length: 32 };
        let mut item = new_default_identity_score("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = DefaultIdentityScorePriorityRangeRule { min: 64, max: 255 };
        let mut item = new_default_identity_score("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
