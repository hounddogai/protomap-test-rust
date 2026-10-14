use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped forecast policy must pass. Check returns its problems, or none.
pub trait CappedForecastPolicyRule: std::fmt::Debug {
    fn check(&self, item: &CappedForecastPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastPolicyAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedForecastPolicyRule for CappedForecastPolicyAllowedCodeRule {
    fn check(&self, item: &CappedForecastPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastPolicyChecksumLengthRule {
    pub max_length: i64,
}

impl CappedForecastPolicyRule for CappedForecastPolicyChecksumLengthRule {
    fn check(&self, item: &CappedForecastPolicy) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastPolicyLabelLengthRule {
    pub max_length: i64,
}

impl CappedForecastPolicyRule for CappedForecastPolicyLabelLengthRule {
    fn check(&self, item: &CappedForecastPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastPolicyCodePrefixRule {
    pub prefix: String,
}

impl CappedForecastPolicyRule for CappedForecastPolicyCodePrefixRule {
    fn check(&self, item: &CappedForecastPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped forecast policy.
#[derive(Debug, Clone)]
pub struct CappedForecastPolicyValidator {
    pub rules: Vec<Rc<dyn CappedForecastPolicyRule>>,
}

impl CappedForecastPolicyValidator {
    pub fn validate(&self, item: &CappedForecastPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedForecastPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedForecastPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_forecast_policy_validator() -> CappedForecastPolicyValidator {
    CappedForecastPolicyValidator {
        rules: vec![
            Rc::new(CappedForecastPolicyAllowedCodeRule {
                allowed: vec!["primary".to_string(), "capped".to_string(), "tiered".to_string()],
            }) as Rc<dyn CappedForecastPolicyRule>,
            Rc::new(CappedForecastPolicyChecksumLengthRule { max_length: 1000 }),
            Rc::new(CappedForecastPolicyLabelLengthRule { max_length: 255 }),
            Rc::new(CappedForecastPolicyCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
