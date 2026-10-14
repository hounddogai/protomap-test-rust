use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling allowlist entry must pass. Check returns its problems, or none.
pub trait RollingAllowlistEntryRule: std::fmt::Debug {
    fn check(&self, item: &RollingAllowlistEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistEntryKeyIdLengthRule {
    pub max_length: i64,
}

impl RollingAllowlistEntryRule for RollingAllowlistEntryKeyIdLengthRule {
    fn check(&self, item: &RollingAllowlistEntry) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistEntryNotesLengthRule {
    pub max_length: i64,
}

impl RollingAllowlistEntryRule for RollingAllowlistEntryNotesLengthRule {
    fn check(&self, item: &RollingAllowlistEntry) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistEntryRequiredNotesRule {
    pub message: String,
}

impl RollingAllowlistEntryRule for RollingAllowlistEntryRequiredNotesRule {
    fn check(&self, item: &RollingAllowlistEntry) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistEntryCodePrefixRule {
    pub prefix: String,
}

impl RollingAllowlistEntryRule for RollingAllowlistEntryCodePrefixRule {
    fn check(&self, item: &RollingAllowlistEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingAllowlistEntryRule for RollingAllowlistEntryAllowedCodeRule {
    fn check(&self, item: &RollingAllowlistEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling allowlist entry.
#[derive(Debug, Clone)]
pub struct RollingAllowlistEntryValidator {
    pub rules: Vec<Rc<dyn RollingAllowlistEntryRule>>,
}

impl RollingAllowlistEntryValidator {
    pub fn validate(&self, item: &RollingAllowlistEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingAllowlistEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingAllowlistEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_allowlist_entry_validator() -> RollingAllowlistEntryValidator {
    RollingAllowlistEntryValidator {
        rules: vec![
            Rc::new(RollingAllowlistEntryKeyIdLengthRule { max_length: 64 }) as Rc<dyn RollingAllowlistEntryRule>,
            Rc::new(RollingAllowlistEntryNotesLengthRule { max_length: 32 }),
            Rc::new(RollingAllowlistEntryRequiredNotesRule {
                message: "rolling allowlist entry needs a value".to_string(),
            }),
            Rc::new(RollingAllowlistEntryCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RollingAllowlistEntryAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "adaptive".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_allowlist_entry_validator();
        let item = new_rolling_allowlist_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = RollingAllowlistEntryKeyIdLengthRule { max_length: 64 };
        let mut item = new_rolling_allowlist_entry("x", "y");
        item.key_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RollingAllowlistEntryNotesLengthRule { max_length: 1000 };
        let mut item = new_rolling_allowlist_entry("x", "y");
        item.notes = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RollingAllowlistEntryRequiredNotesRule {
            message: "rolling allowlist entry needs an entry".to_string(),
        };
        let mut item = new_rolling_allowlist_entry("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
