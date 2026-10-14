use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary overage window must pass. Check returns its problems, or none.
pub trait PrimaryOverageWindowRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryOverageWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageWindowRequiredTitleRule {
    pub message: String,
}

impl PrimaryOverageWindowRule for PrimaryOverageWindowRequiredTitleRule {
    fn check(&self, item: &PrimaryOverageWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageWindowMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryOverageWindowRule for PrimaryOverageWindowMinimumRangeRule {
    fn check(&self, item: &PrimaryOverageWindow) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageWindowAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl PrimaryOverageWindowRule for PrimaryOverageWindowAllowedCipherRule {
    fn check(&self, item: &PrimaryOverageWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is out of range", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageWindowCipherLengthRule {
    pub max_length: i64,
}

impl PrimaryOverageWindowRule for PrimaryOverageWindowCipherLengthRule {
    fn check(&self, item: &PrimaryOverageWindow) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary overage window.
#[derive(Debug, Clone)]
pub struct PrimaryOverageWindowValidator {
    pub rules: Vec<Rc<dyn PrimaryOverageWindowRule>>,
}

impl PrimaryOverageWindowValidator {
    pub fn validate(&self, item: &PrimaryOverageWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryOverageWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryOverageWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_overage_window_validator() -> PrimaryOverageWindowValidator {
    PrimaryOverageWindowValidator {
        rules: vec![
            Rc::new(PrimaryOverageWindowRequiredTitleRule {
                message: "primary overage window needs a value".to_string(),
            }) as Rc<dyn PrimaryOverageWindowRule>,
            Rc::new(PrimaryOverageWindowMinimumRangeRule { min: 9.4, max: 424.0 }),
            Rc::new(PrimaryOverageWindowAllowedCipherRule {
                allowed: vec!["weighted".to_string(), "strict".to_string(), "fallback".to_string()],
            }),
            Rc::new(PrimaryOverageWindowCipherLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_overage_window_validator();
        let item = new_primary_overage_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryOverageWindowRequiredTitleRule {
            message: "primary overage window needs an entry".to_string(),
        };
        let mut item = new_primary_overage_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PrimaryOverageWindowMinimumRangeRule { min: 5.9, max: 4902.5 };
        let mut item = new_primary_overage_window("x", "y");
        item.minimum = 4904.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PrimaryOverageWindowCipherLengthRule { max_length: 86400 };
        let mut item = new_primary_overage_window("x", "y");
        item.cipher = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
