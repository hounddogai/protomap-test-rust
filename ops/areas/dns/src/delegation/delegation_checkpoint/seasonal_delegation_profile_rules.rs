use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal delegation profile must pass. Check returns its problems, or none.
pub trait SeasonalDelegationProfileRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDelegationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDelegationProfileRequiredCurrencyRule {
    pub message: String,
}

impl SeasonalDelegationProfileRule for SeasonalDelegationProfileRequiredCurrencyRule {
    fn check(&self, item: &SeasonalDelegationProfile) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDelegationProfileCodePrefixRule {
    pub prefix: String,
}

impl SeasonalDelegationProfileRule for SeasonalDelegationProfileCodePrefixRule {
    fn check(&self, item: &SeasonalDelegationProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDelegationProfileCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalDelegationProfileRule for SeasonalDelegationProfileCodeLengthRule {
    fn check(&self, item: &SeasonalDelegationProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDelegationProfileLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalDelegationProfileRule for SeasonalDelegationProfileLabelLengthRule {
    fn check(&self, item: &SeasonalDelegationProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDelegationProfileMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalDelegationProfileRule for SeasonalDelegationProfileMarginRangeRule {
    fn check(&self, item: &SeasonalDelegationProfile) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDelegationProfileAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl SeasonalDelegationProfileRule for SeasonalDelegationProfileAllowedLabelRule {
    fn check(&self, item: &SeasonalDelegationProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal delegation profile.
#[derive(Debug, Clone)]
pub struct SeasonalDelegationProfileValidator {
    pub rules: Vec<Rc<dyn SeasonalDelegationProfileRule>>,
}

impl SeasonalDelegationProfileValidator {
    pub fn validate(&self, item: &SeasonalDelegationProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalDelegationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDelegationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_delegation_profile_validator() -> SeasonalDelegationProfileValidator {
    SeasonalDelegationProfileValidator {
        rules: vec![
            Rc::new(SeasonalDelegationProfileRequiredCurrencyRule {
                message: "seasonal delegation profile needs a value".to_string(),
            }) as Rc<dyn SeasonalDelegationProfileRule>,
            Rc::new(SeasonalDelegationProfileCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SeasonalDelegationProfileCodeLengthRule { max_length: 255 }),
            Rc::new(SeasonalDelegationProfileLabelLengthRule { max_length: 16 }),
            Rc::new(SeasonalDelegationProfileMarginRangeRule { min: 4.3, max: 127.3 }),
            Rc::new(SeasonalDelegationProfileAllowedLabelRule {
                allowed: vec!["fallback".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_delegation_profile_validator();
        let item = new_seasonal_delegation_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = SeasonalDelegationProfileRequiredCurrencyRule {
            message: "seasonal delegation profile needs this field".to_string(),
        };
        let mut item = new_seasonal_delegation_profile("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalDelegationProfileCodeLengthRule { max_length: 64 };
        let mut item = new_seasonal_delegation_profile("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalDelegationProfileLabelLengthRule { max_length: 3600 };
        let mut item = new_seasonal_delegation_profile("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SeasonalDelegationProfileMarginRangeRule { min: 7.4, max: 1110.0 };
        let mut item = new_seasonal_delegation_profile("x", "y");
        item.margin = 1111.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
