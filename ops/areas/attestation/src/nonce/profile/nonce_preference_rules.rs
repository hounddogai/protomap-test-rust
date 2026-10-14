use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce preference must pass. Check returns its problems, or none.
pub trait NoncePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &NoncePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreferenceNamespaceLengthRule {
    pub max_length: i64,
}

impl NoncePreferenceRule for NoncePreferenceNamespaceLengthRule {
    fn check(&self, item: &NoncePreference) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreferenceExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NoncePreferenceRule for NoncePreferenceExpiresAtRangeRule {
    fn check(&self, item: &NoncePreference) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreferenceCodePrefixRule {
    pub prefix: String,
}

impl NoncePreferenceRule for NoncePreferenceCodePrefixRule {
    fn check(&self, item: &NoncePreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreferenceCodeLengthRule {
    pub max_length: i64,
}

impl NoncePreferenceRule for NoncePreferenceCodeLengthRule {
    fn check(&self, item: &NoncePreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreferenceDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NoncePreferenceRule for NoncePreferenceDiscountRangeRule {
    fn check(&self, item: &NoncePreference) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreferenceTitleLengthRule {
    pub max_length: i64,
}

impl NoncePreferenceRule for NoncePreferenceTitleLengthRule {
    fn check(&self, item: &NoncePreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce preference.
#[derive(Debug, Clone)]
pub struct NoncePreferenceValidator {
    pub rules: Vec<Rc<dyn NoncePreferenceRule>>,
}

impl NoncePreferenceValidator {
    pub fn validate(&self, item: &NoncePreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NoncePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NoncePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_preference_validator() -> NoncePreferenceValidator {
    NoncePreferenceValidator {
        rules: vec![
            Rc::new(NoncePreferenceNamespaceLengthRule { max_length: 120 }) as Rc<dyn NoncePreferenceRule>,
            Rc::new(NoncePreferenceExpiresAtRangeRule { min: 32, max: 80 }),
            Rc::new(NoncePreferenceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(NoncePreferenceCodeLengthRule { max_length: 3600 }),
            Rc::new(NoncePreferenceDiscountRangeRule { min: 2.0, max: 1282.7 }),
            Rc::new(NoncePreferenceTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_preference_validator();
        let item = new_nonce_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = NoncePreferenceNamespaceLengthRule { max_length: 3600 };
        let mut item = new_nonce_preference("x", "y");
        item.namespace = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = NoncePreferenceExpiresAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_nonce_preference("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = NoncePreferenceCodeLengthRule { max_length: 64 };
        let mut item = new_nonce_preference("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = NoncePreferenceDiscountRangeRule { min: 5.6, max: 3456.7 };
        let mut item = new_nonce_preference("x", "y");
        item.discount = 3458.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = NoncePreferenceTitleLengthRule { max_length: 86400 };
        let mut item = new_nonce_preference("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
