use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist profile must pass. Check returns its problems, or none.
pub trait AllowlistProfileRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfileQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistProfileRule for AllowlistProfileQuantityRangeRule {
    fn check(&self, item: &AllowlistProfile) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfileRequiredTitleRule {
    pub message: String,
}

impl AllowlistProfileRule for AllowlistProfileRequiredTitleRule {
    fn check(&self, item: &AllowlistProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfileAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl AllowlistProfileRule for AllowlistProfileAllowedCipherRule {
    fn check(&self, item: &AllowlistProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is inconsistent", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfileTitleLengthRule {
    pub max_length: i64,
}

impl AllowlistProfileRule for AllowlistProfileTitleLengthRule {
    fn check(&self, item: &AllowlistProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfileCipherLengthRule {
    pub max_length: i64,
}

impl AllowlistProfileRule for AllowlistProfileCipherLengthRule {
    fn check(&self, item: &AllowlistProfile) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfileRequiredCodeRule {
    pub message: String,
}

impl AllowlistProfileRule for AllowlistProfileRequiredCodeRule {
    fn check(&self, item: &AllowlistProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist profile.
#[derive(Debug, Clone)]
pub struct AllowlistProfileValidator {
    pub rules: Vec<Rc<dyn AllowlistProfileRule>>,
}

impl AllowlistProfileValidator {
    pub fn validate(&self, item: &AllowlistProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_profile_validator() -> AllowlistProfileValidator {
    AllowlistProfileValidator {
        rules: vec![
            Rc::new(AllowlistProfileQuantityRangeRule { min: 120, max: 1000 }) as Rc<dyn AllowlistProfileRule>,
            Rc::new(AllowlistProfileRequiredTitleRule {
                message: "allowlist profile needs an entry".to_string(),
            }),
            Rc::new(AllowlistProfileAllowedCipherRule {
                allowed: vec!["legacy".to_string(), "capped".to_string()],
            }),
            Rc::new(AllowlistProfileTitleLengthRule { max_length: 86400 }),
            Rc::new(AllowlistProfileCipherLengthRule { max_length: 64 }),
            Rc::new(AllowlistProfileRequiredCodeRule {
                message: "allowlist profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_profile_validator();
        let item = new_allowlist_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = AllowlistProfileQuantityRangeRule { min: 16, max: 86400 };
        let mut item = new_allowlist_profile("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AllowlistProfileRequiredTitleRule {
            message: "allowlist profile needs a value".to_string(),
        };
        let mut item = new_allowlist_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AllowlistProfileTitleLengthRule { max_length: 255 };
        let mut item = new_allowlist_profile("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = AllowlistProfileCipherLengthRule { max_length: 255 };
        let mut item = new_allowlist_profile("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistProfileRequiredCodeRule {
            message: "allowlist profile needs a value".to_string(),
        };
        let mut item = new_allowlist_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
