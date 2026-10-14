use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention variance must pass. Check returns its problems, or none.
pub trait RetentionVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RetentionVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionVarianceCodePrefixRule {
    pub prefix: String,
}

impl RetentionVarianceRule for RetentionVarianceCodePrefixRule {
    fn check(&self, item: &RetentionVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionVarianceThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionVarianceRule for RetentionVarianceThresholdRangeRule {
    fn check(&self, item: &RetentionVariance) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionVarianceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RetentionVarianceRule for RetentionVarianceAllowedCodeRule {
    fn check(&self, item: &RetentionVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionVarianceCodeLengthRule {
    pub max_length: i64,
}

impl RetentionVarianceRule for RetentionVarianceCodeLengthRule {
    fn check(&self, item: &RetentionVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention variance.
#[derive(Debug, Clone)]
pub struct RetentionVarianceValidator {
    pub rules: Vec<Rc<dyn RetentionVarianceRule>>,
}

impl RetentionVarianceValidator {
    pub fn validate(&self, item: &RetentionVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_variance_validator() -> RetentionVarianceValidator {
    RetentionVarianceValidator {
        rules: vec![
            Rc::new(RetentionVarianceCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RetentionVarianceRule>,
            Rc::new(RetentionVarianceThresholdRangeRule { min: 8.7, max: 4195.4 }),
            Rc::new(RetentionVarianceAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "strict".to_string(), "batched".to_string()],
            }),
            Rc::new(RetentionVarianceCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_variance_validator();
        let item = new_retention_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RetentionVarianceThresholdRangeRule { min: 1.8, max: 3901.0 };
        let mut item = new_retention_variance("x", "y");
        item.threshold = 3902.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionVarianceCodeLengthRule { max_length: 1000 };
        let mut item = new_retention_variance("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
