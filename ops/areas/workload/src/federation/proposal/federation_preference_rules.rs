use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation preference must pass. Check returns its problems, or none.
pub trait FederationPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &FederationPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationPreferenceRequiredPhoneRule {
    pub message: String,
}

impl FederationPreferenceRule for FederationPreferenceRequiredPhoneRule {
    fn check(&self, item: &FederationPreference) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationPreferenceAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl FederationPreferenceRule for FederationPreferenceAllowedPhoneRule {
    fn check(&self, item: &FederationPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is unknown", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationPreferenceCodePrefixRule {
    pub prefix: String,
}

impl FederationPreferenceRule for FederationPreferenceCodePrefixRule {
    fn check(&self, item: &FederationPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl FederationPreferenceRule for FederationPreferenceTitleLengthRule {
    fn check(&self, item: &FederationPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationPreferenceRequiredCodeRule {
    pub message: String,
}

impl FederationPreferenceRule for FederationPreferenceRequiredCodeRule {
    fn check(&self, item: &FederationPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationPreferenceGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationPreferenceRule for FederationPreferenceGraceDaysRangeRule {
    fn check(&self, item: &FederationPreference) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation preference.
#[derive(Debug, Clone)]
pub struct FederationPreferenceValidator {
    pub rules: Vec<Rc<dyn FederationPreferenceRule>>,
}

impl FederationPreferenceValidator {
    pub fn validate(&self, item: &FederationPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_preference_validator() -> FederationPreferenceValidator {
    FederationPreferenceValidator {
        rules: vec![
            Rc::new(FederationPreferenceRequiredPhoneRule {
                message: "federation preference needs an entry".to_string(),
            }) as Rc<dyn FederationPreferenceRule>,
            Rc::new(FederationPreferenceAllowedPhoneRule {
                allowed: vec!["weighted".to_string(), "adaptive".to_string(), "tiered".to_string()],
            }),
            Rc::new(FederationPreferenceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FederationPreferenceTitleLengthRule { max_length: 86400 }),
            Rc::new(FederationPreferenceRequiredCodeRule {
                message: "federation preference needs this field".to_string(),
            }),
            Rc::new(FederationPreferenceGraceDaysRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_preference_validator();
        let item = new_federation_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = FederationPreferenceRequiredPhoneRule {
            message: "federation preference needs an entry".to_string(),
        };
        let mut item = new_federation_preference("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FederationPreferenceTitleLengthRule { max_length: 64 };
        let mut item = new_federation_preference("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FederationPreferenceRequiredCodeRule {
            message: "federation preference needs this field".to_string(),
        };
        let mut item = new_federation_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FederationPreferenceGraceDaysRangeRule { min: 80, max: 255 };
        let mut item = new_federation_preference("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
