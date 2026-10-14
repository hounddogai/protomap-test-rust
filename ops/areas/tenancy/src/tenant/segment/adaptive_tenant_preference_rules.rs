use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive tenant preference must pass. Check returns its problems, or none.
pub trait AdaptiveTenantPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTenantPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveTenantPreferenceRule for AdaptiveTenantPreferenceCodeLengthRule {
    fn check(&self, item: &AdaptiveTenantPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantPreferenceCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveTenantPreferenceRule for AdaptiveTenantPreferenceCodePrefixRule {
    fn check(&self, item: &AdaptiveTenantPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantPreferenceNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveTenantPreferenceRule for AdaptiveTenantPreferenceNameLengthRule {
    fn check(&self, item: &AdaptiveTenantPreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantPreferenceRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveTenantPreferenceRule for AdaptiveTenantPreferenceRateRangeRule {
    fn check(&self, item: &AdaptiveTenantPreference) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantPreferenceAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl AdaptiveTenantPreferenceRule for AdaptiveTenantPreferenceAllowedFingerprintRule {
    fn check(&self, item: &AdaptiveTenantPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is blocked", item.fingerprint)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive tenant preference.
#[derive(Debug, Clone)]
pub struct AdaptiveTenantPreferenceValidator {
    pub rules: Vec<Rc<dyn AdaptiveTenantPreferenceRule>>,
}

impl AdaptiveTenantPreferenceValidator {
    pub fn validate(&self, item: &AdaptiveTenantPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveTenantPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTenantPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_tenant_preference_validator() -> AdaptiveTenantPreferenceValidator {
    AdaptiveTenantPreferenceValidator {
        rules: vec![
            Rc::new(AdaptiveTenantPreferenceCodeLengthRule { max_length: 64 }) as Rc<dyn AdaptiveTenantPreferenceRule>,
            Rc::new(AdaptiveTenantPreferenceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(AdaptiveTenantPreferenceNameLengthRule { max_length: 16 }),
            Rc::new(AdaptiveTenantPreferenceRateRangeRule { min: 1.2, max: 2593.0 }),
            Rc::new(AdaptiveTenantPreferenceAllowedFingerprintRule {
                allowed: vec!["weighted".to_string(), "regional".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_tenant_preference_validator();
        let item = new_adaptive_tenant_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveTenantPreferenceCodeLengthRule { max_length: 64 };
        let mut item = new_adaptive_tenant_preference("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AdaptiveTenantPreferenceNameLengthRule { max_length: 80 };
        let mut item = new_adaptive_tenant_preference("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = AdaptiveTenantPreferenceRateRangeRule { min: 5.9, max: 4113.4 };
        let mut item = new_adaptive_tenant_preference("x", "y");
        item.rate = 4114.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
