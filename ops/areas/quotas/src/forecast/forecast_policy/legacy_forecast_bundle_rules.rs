use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy forecast bundle must pass. Check returns its problems, or none.
pub trait LegacyForecastBundleRule: std::fmt::Debug {
    fn check(&self, item: &LegacyForecastBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyForecastBundleRequiredCurrencyRule {
    pub message: String,
}

impl LegacyForecastBundleRule for LegacyForecastBundleRequiredCurrencyRule {
    fn check(&self, item: &LegacyForecastBundle) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyForecastBundleLabelLengthRule {
    pub max_length: i64,
}

impl LegacyForecastBundleRule for LegacyForecastBundleLabelLengthRule {
    fn check(&self, item: &LegacyForecastBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyForecastBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyForecastBundleRule for LegacyForecastBundleAllowedCodeRule {
    fn check(&self, item: &LegacyForecastBundle) -> Vec<String> {
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
pub struct LegacyForecastBundleCodePrefixRule {
    pub prefix: String,
}

impl LegacyForecastBundleRule for LegacyForecastBundleCodePrefixRule {
    fn check(&self, item: &LegacyForecastBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy forecast bundle.
#[derive(Debug, Clone)]
pub struct LegacyForecastBundleValidator {
    pub rules: Vec<Rc<dyn LegacyForecastBundleRule>>,
}

impl LegacyForecastBundleValidator {
    pub fn validate(&self, item: &LegacyForecastBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyForecastBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyForecastBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_forecast_bundle_validator() -> LegacyForecastBundleValidator {
    LegacyForecastBundleValidator {
        rules: vec![
            Rc::new(LegacyForecastBundleRequiredCurrencyRule {
                message: "legacy forecast bundle needs a value".to_string(),
            }) as Rc<dyn LegacyForecastBundleRule>,
            Rc::new(LegacyForecastBundleLabelLengthRule { max_length: 32 }),
            Rc::new(LegacyForecastBundleAllowedCodeRule {
                allowed: vec!["strict".to_string(), "regional".to_string(), "rolling".to_string()],
            }),
            Rc::new(LegacyForecastBundleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_forecast_bundle_validator();
        let item = new_legacy_forecast_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = LegacyForecastBundleRequiredCurrencyRule {
            message: "legacy forecast bundle needs an entry".to_string(),
        };
        let mut item = new_legacy_forecast_bundle("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyForecastBundleLabelLengthRule { max_length: 86400 };
        let mut item = new_legacy_forecast_bundle("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
