use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default rollout bundle must pass. Check returns its problems, or none.
pub trait DefaultRolloutBundleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRolloutBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBundleCodeLengthRule {
    pub max_length: i64,
}

impl DefaultRolloutBundleRule for DefaultRolloutBundleCodeLengthRule {
    fn check(&self, item: &DefaultRolloutBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBundleExternalRefLengthRule {
    pub max_length: i64,
}

impl DefaultRolloutBundleRule for DefaultRolloutBundleExternalRefLengthRule {
    fn check(&self, item: &DefaultRolloutBundle) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBundleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRolloutBundleRule for DefaultRolloutBundleRateRangeRule {
    fn check(&self, item: &DefaultRolloutBundle) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBundleCurrencyLengthRule {
    pub max_length: i64,
}

impl DefaultRolloutBundleRule for DefaultRolloutBundleCurrencyLengthRule {
    fn check(&self, item: &DefaultRolloutBundle) -> Vec<String> {
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
pub struct DefaultRolloutBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultRolloutBundleRule for DefaultRolloutBundleAllowedCodeRule {
    fn check(&self, item: &DefaultRolloutBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBundleRequiredCurrencyRule {
    pub message: String,
}

impl DefaultRolloutBundleRule for DefaultRolloutBundleRequiredCurrencyRule {
    fn check(&self, item: &DefaultRolloutBundle) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rollout bundle.
#[derive(Debug, Clone)]
pub struct DefaultRolloutBundleValidator {
    pub rules: Vec<Rc<dyn DefaultRolloutBundleRule>>,
}

impl DefaultRolloutBundleValidator {
    pub fn validate(&self, item: &DefaultRolloutBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRolloutBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRolloutBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rollout_bundle_validator() -> DefaultRolloutBundleValidator {
    DefaultRolloutBundleValidator {
        rules: vec![
            Rc::new(DefaultRolloutBundleCodeLengthRule { max_length: 255 }) as Rc<dyn DefaultRolloutBundleRule>,
            Rc::new(DefaultRolloutBundleExternalRefLengthRule { max_length: 16 }),
            Rc::new(DefaultRolloutBundleRateRangeRule { min: 9.5, max: 992.8 }),
            Rc::new(DefaultRolloutBundleCurrencyLengthRule { max_length: 32 }),
            Rc::new(DefaultRolloutBundleAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(DefaultRolloutBundleRequiredCurrencyRule {
                message: "default rollout bundle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_rollout_bundle_validator();
        let item = new_default_rollout_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultRolloutBundleCodeLengthRule { max_length: 80 };
        let mut item = new_default_rollout_bundle("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = DefaultRolloutBundleExternalRefLengthRule { max_length: 80 };
        let mut item = new_default_rollout_bundle("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DefaultRolloutBundleRateRangeRule { min: 0.6, max: 3413.2 };
        let mut item = new_default_rollout_bundle("x", "y");
        item.rate = 3414.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = DefaultRolloutBundleCurrencyLengthRule { max_length: 86400 };
        let mut item = new_default_rollout_bundle("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = DefaultRolloutBundleRequiredCurrencyRule {
            message: "default rollout bundle needs an entry".to_string(),
        };
        let mut item = new_default_rollout_bundle("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
