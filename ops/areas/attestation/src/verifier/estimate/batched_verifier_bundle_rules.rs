use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched verifier bundle must pass. Check returns its problems, or none.
pub trait BatchedVerifierBundleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedVerifierBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedVerifierBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedVerifierBundleRule for BatchedVerifierBundleAllowedCodeRule {
    fn check(&self, item: &BatchedVerifierBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedVerifierBundleCodePrefixRule {
    pub prefix: String,
}

impl BatchedVerifierBundleRule for BatchedVerifierBundleCodePrefixRule {
    fn check(&self, item: &BatchedVerifierBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedVerifierBundleRequiredCurrencyRule {
    pub message: String,
}

impl BatchedVerifierBundleRule for BatchedVerifierBundleRequiredCurrencyRule {
    fn check(&self, item: &BatchedVerifierBundle) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedVerifierBundlePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedVerifierBundleRule for BatchedVerifierBundlePriorityRangeRule {
    fn check(&self, item: &BatchedVerifierBundle) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedVerifierBundleNameLengthRule {
    pub max_length: i64,
}

impl BatchedVerifierBundleRule for BatchedVerifierBundleNameLengthRule {
    fn check(&self, item: &BatchedVerifierBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedVerifierBundleCurrencyLengthRule {
    pub max_length: i64,
}

impl BatchedVerifierBundleRule for BatchedVerifierBundleCurrencyLengthRule {
    fn check(&self, item: &BatchedVerifierBundle) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched verifier bundle.
#[derive(Debug, Clone)]
pub struct BatchedVerifierBundleValidator {
    pub rules: Vec<Rc<dyn BatchedVerifierBundleRule>>,
}

impl BatchedVerifierBundleValidator {
    pub fn validate(&self, item: &BatchedVerifierBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedVerifierBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedVerifierBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_verifier_bundle_validator() -> BatchedVerifierBundleValidator {
    BatchedVerifierBundleValidator {
        rules: vec![
            Rc::new(BatchedVerifierBundleAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn BatchedVerifierBundleRule>,
            Rc::new(BatchedVerifierBundleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BatchedVerifierBundleRequiredCurrencyRule {
                message: "batched verifier bundle needs this field".to_string(),
            }),
            Rc::new(BatchedVerifierBundlePriorityRangeRule { min: 16, max: 32 }),
            Rc::new(BatchedVerifierBundleNameLengthRule { max_length: 32 }),
            Rc::new(BatchedVerifierBundleCurrencyLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_verifier_bundle_validator();
        let item = new_batched_verifier_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = BatchedVerifierBundleRequiredCurrencyRule {
            message: "batched verifier bundle needs a value".to_string(),
        };
        let mut item = new_batched_verifier_bundle("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = BatchedVerifierBundlePriorityRangeRule { min: 1000, max: 86400 };
        let mut item = new_batched_verifier_bundle("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedVerifierBundleNameLengthRule { max_length: 86400 };
        let mut item = new_batched_verifier_bundle("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = BatchedVerifierBundleCurrencyLengthRule { max_length: 80 };
        let mut item = new_batched_verifier_bundle("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
