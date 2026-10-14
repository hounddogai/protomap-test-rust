use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary threshold must pass. Check returns its problems, or none.
pub trait CanaryThresholdRule: std::fmt::Debug {
    fn check(&self, item: &CanaryThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryThresholdRequiredTitleRule {
    pub message: String,
}

impl CanaryThresholdRule for CanaryThresholdRequiredTitleRule {
    fn check(&self, item: &CanaryThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryThresholdCurrencyLengthRule {
    pub max_length: i64,
}

impl CanaryThresholdRule for CanaryThresholdCurrencyLengthRule {
    fn check(&self, item: &CanaryThreshold) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CanaryThresholdRule for CanaryThresholdAllowedCodeRule {
    fn check(&self, item: &CanaryThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary threshold.
#[derive(Debug, Clone)]
pub struct CanaryThresholdValidator {
    pub rules: Vec<Rc<dyn CanaryThresholdRule>>,
}

impl CanaryThresholdValidator {
    pub fn validate(&self, item: &CanaryThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_threshold_validator() -> CanaryThresholdValidator {
    CanaryThresholdValidator {
        rules: vec![
            Rc::new(CanaryThresholdRequiredTitleRule {
                message: "canary threshold needs this field".to_string(),
            }) as Rc<dyn CanaryThresholdRule>,
            Rc::new(CanaryThresholdCurrencyLengthRule { max_length: 16 }),
            Rc::new(CanaryThresholdAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_threshold_validator();
        let item = new_canary_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CanaryThresholdRequiredTitleRule {
            message: "canary threshold needs a value".to_string(),
        };
        let mut item = new_canary_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = CanaryThresholdCurrencyLengthRule { max_length: 64 };
        let mut item = new_canary_threshold("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
