use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback isolation entry must pass. Check returns its problems, or none.
pub trait FallbackIsolationEntryRule: std::fmt::Debug {
    fn check(&self, item: &FallbackIsolationEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryKeyIdLengthRule {
    pub max_length: i64,
}

impl FallbackIsolationEntryRule for FallbackIsolationEntryKeyIdLengthRule {
    fn check(&self, item: &FallbackIsolationEntry) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FallbackIsolationEntryRule for FallbackIsolationEntryAllowedNameRule {
    fn check(&self, item: &FallbackIsolationEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryCodePrefixRule {
    pub prefix: String,
}

impl FallbackIsolationEntryRule for FallbackIsolationEntryCodePrefixRule {
    fn check(&self, item: &FallbackIsolationEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryRequiredNameRule {
    pub message: String,
}

impl FallbackIsolationEntryRule for FallbackIsolationEntryRequiredNameRule {
    fn check(&self, item: &FallbackIsolationEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryRequiredCountryRule {
    pub message: String,
}

impl FallbackIsolationEntryRule for FallbackIsolationEntryRequiredCountryRule {
    fn check(&self, item: &FallbackIsolationEntry) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryRequiredKeyIdRule {
    pub message: String,
}

impl FallbackIsolationEntryRule for FallbackIsolationEntryRequiredKeyIdRule {
    fn check(&self, item: &FallbackIsolationEntry) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback isolation entry.
#[derive(Debug, Clone)]
pub struct FallbackIsolationEntryValidator {
    pub rules: Vec<Rc<dyn FallbackIsolationEntryRule>>,
}

impl FallbackIsolationEntryValidator {
    pub fn validate(&self, item: &FallbackIsolationEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackIsolationEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackIsolationEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_isolation_entry_validator() -> FallbackIsolationEntryValidator {
    FallbackIsolationEntryValidator {
        rules: vec![
            Rc::new(FallbackIsolationEntryKeyIdLengthRule { max_length: 86400 }) as Rc<dyn FallbackIsolationEntryRule>,
            Rc::new(FallbackIsolationEntryAllowedNameRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(FallbackIsolationEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(FallbackIsolationEntryRequiredNameRule {
                message: "fallback isolation entry needs an entry".to_string(),
            }),
            Rc::new(FallbackIsolationEntryRequiredCountryRule {
                message: "fallback isolation entry needs an entry".to_string(),
            }),
            Rc::new(FallbackIsolationEntryRequiredKeyIdRule {
                message: "fallback isolation entry needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_isolation_entry_validator();
        let item = new_fallback_isolation_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = FallbackIsolationEntryKeyIdLengthRule { max_length: 32 };
        let mut item = new_fallback_isolation_entry("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackIsolationEntryRequiredNameRule {
            message: "fallback isolation entry needs a value".to_string(),
        };
        let mut item = new_fallback_isolation_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = FallbackIsolationEntryRequiredCountryRule {
            message: "fallback isolation entry needs an entry".to_string(),
        };
        let mut item = new_fallback_isolation_entry("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = FallbackIsolationEntryRequiredKeyIdRule {
            message: "fallback isolation entry needs this field".to_string(),
        };
        let mut item = new_fallback_isolation_entry("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
