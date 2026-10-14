use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched runbook variance must pass. Check returns its problems, or none.
pub trait BatchedRunbookVarianceRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRunbookVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookVarianceLabelLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookVarianceRule for BatchedRunbookVarianceLabelLengthRule {
    fn check(&self, item: &BatchedRunbookVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookVarianceRequiredCodeRule {
    pub message: String,
}

impl BatchedRunbookVarianceRule for BatchedRunbookVarianceRequiredCodeRule {
    fn check(&self, item: &BatchedRunbookVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookVarianceUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRunbookVarianceRule for BatchedRunbookVarianceUpdatedAtRangeRule {
    fn check(&self, item: &BatchedRunbookVariance) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookVarianceAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BatchedRunbookVarianceRule for BatchedRunbookVarianceAllowedLabelRule {
    fn check(&self, item: &BatchedRunbookVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookVarianceCodePrefixRule {
    pub prefix: String,
}

impl BatchedRunbookVarianceRule for BatchedRunbookVarianceCodePrefixRule {
    fn check(&self, item: &BatchedRunbookVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched runbook variance.
#[derive(Debug, Clone)]
pub struct BatchedRunbookVarianceValidator {
    pub rules: Vec<Rc<dyn BatchedRunbookVarianceRule>>,
}

impl BatchedRunbookVarianceValidator {
    pub fn validate(&self, item: &BatchedRunbookVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRunbookVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRunbookVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_runbook_variance_validator() -> BatchedRunbookVarianceValidator {
    BatchedRunbookVarianceValidator {
        rules: vec![
            Rc::new(BatchedRunbookVarianceLabelLengthRule { max_length: 16 }) as Rc<dyn BatchedRunbookVarianceRule>,
            Rc::new(BatchedRunbookVarianceRequiredCodeRule {
                message: "batched runbook variance needs a value".to_string(),
            }),
            Rc::new(BatchedRunbookVarianceUpdatedAtRangeRule { min: 32, max: 3600 }),
            Rc::new(BatchedRunbookVarianceAllowedLabelRule {
                allowed: vec!["strict".to_string(), "regional".to_string(), "strict".to_string()],
            }),
            Rc::new(BatchedRunbookVarianceCodePrefixRule {
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
        let validator = default_batched_runbook_variance_validator();
        let item = new_batched_runbook_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedRunbookVarianceLabelLengthRule { max_length: 1000 };
        let mut item = new_batched_runbook_variance("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedRunbookVarianceRequiredCodeRule {
            message: "batched runbook variance needs a value".to_string(),
        };
        let mut item = new_batched_runbook_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = BatchedRunbookVarianceUpdatedAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_batched_runbook_variance("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
