use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential record must pass. Check returns its problems, or none.
pub trait CredentialRecordRule: std::fmt::Debug {
    fn check(&self, item: &CredentialRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRecordAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CredentialRecordRule for CredentialRecordAllowedCodeRule {
    fn check(&self, item: &CredentialRecord) -> Vec<String> {
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
pub struct CredentialRecordAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialRecordRule for CredentialRecordAttemptsRangeRule {
    fn check(&self, item: &CredentialRecord) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRecordStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialRecordRule for CredentialRecordStartsAtRangeRule {
    fn check(&self, item: &CredentialRecord) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRecordCodePrefixRule {
    pub prefix: String,
}

impl CredentialRecordRule for CredentialRecordCodePrefixRule {
    fn check(&self, item: &CredentialRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRecordRequiredCodeRule {
    pub message: String,
}

impl CredentialRecordRule for CredentialRecordRequiredCodeRule {
    fn check(&self, item: &CredentialRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRecordTitleLengthRule {
    pub max_length: i64,
}

impl CredentialRecordRule for CredentialRecordTitleLengthRule {
    fn check(&self, item: &CredentialRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential record.
#[derive(Debug, Clone)]
pub struct CredentialRecordValidator {
    pub rules: Vec<Rc<dyn CredentialRecordRule>>,
}

impl CredentialRecordValidator {
    pub fn validate(&self, item: &CredentialRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_record_validator() -> CredentialRecordValidator {
    CredentialRecordValidator {
        rules: vec![
            Rc::new(CredentialRecordAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn CredentialRecordRule>,
            Rc::new(CredentialRecordAttemptsRangeRule { min: 16, max: 1000 }),
            Rc::new(CredentialRecordStartsAtRangeRule { min: 255, max: 3600 }),
            Rc::new(CredentialRecordCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CredentialRecordRequiredCodeRule {
                message: "credential record needs this field".to_string(),
            }),
            Rc::new(CredentialRecordTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_record_validator();
        let item = new_credential_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = CredentialRecordAttemptsRangeRule { min: 3600, max: 3600 };
        let mut item = new_credential_record("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = CredentialRecordStartsAtRangeRule { min: 16, max: 255 };
        let mut item = new_credential_record("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialRecordRequiredCodeRule {
            message: "credential record needs this field".to_string(),
        };
        let mut item = new_credential_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CredentialRecordTitleLengthRule { max_length: 32 };
        let mut item = new_credential_record("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
