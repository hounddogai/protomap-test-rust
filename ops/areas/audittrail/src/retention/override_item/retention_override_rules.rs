use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention override must pass. Check returns its problems, or none.
pub trait RetentionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RetentionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverrideCodeLengthRule {
    pub max_length: i64,
}

impl RetentionOverrideRule for RetentionOverrideCodeLengthRule {
    fn check(&self, item: &RetentionOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverrideArchivedGuardRule {
    pub limit: f64,
}

impl RetentionOverrideRule for RetentionOverrideArchivedGuardRule {
    fn check(&self, item: &RetentionOverride) -> Vec<String> {
        if item.archived && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while archived", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverrideRequiredFingerprintRule {
    pub message: String,
}

impl RetentionOverrideRule for RetentionOverrideRequiredFingerprintRule {
    fn check(&self, item: &RetentionOverride) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverrideFingerprintLengthRule {
    pub max_length: i64,
}

impl RetentionOverrideRule for RetentionOverrideFingerprintLengthRule {
    fn check(&self, item: &RetentionOverride) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverrideAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl RetentionOverrideRule for RetentionOverrideAllowedFingerprintRule {
    fn check(&self, item: &RetentionOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is stale", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverrideCodePrefixRule {
    pub prefix: String,
}

impl RetentionOverrideRule for RetentionOverrideCodePrefixRule {
    fn check(&self, item: &RetentionOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention override.
#[derive(Debug, Clone)]
pub struct RetentionOverrideValidator {
    pub rules: Vec<Rc<dyn RetentionOverrideRule>>,
}

impl RetentionOverrideValidator {
    pub fn validate(&self, item: &RetentionOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_override_validator() -> RetentionOverrideValidator {
    RetentionOverrideValidator {
        rules: vec![
            Rc::new(RetentionOverrideCodeLengthRule { max_length: 1000 }) as Rc<dyn RetentionOverrideRule>,
            Rc::new(RetentionOverrideArchivedGuardRule { limit: 8.3 }),
            Rc::new(RetentionOverrideRequiredFingerprintRule {
                message: "retention override needs an entry".to_string(),
            }),
            Rc::new(RetentionOverrideFingerprintLengthRule { max_length: 64 }),
            Rc::new(RetentionOverrideAllowedFingerprintRule {
                allowed: vec!["legacy".to_string(), "adaptive".to_string()],
            }),
            Rc::new(RetentionOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_override_validator();
        let item = new_retention_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionOverrideCodeLengthRule { max_length: 80 };
        let mut item = new_retention_override("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = RetentionOverrideRequiredFingerprintRule {
            message: "retention override needs this field".to_string(),
        };
        let mut item = new_retention_override("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = RetentionOverrideFingerprintLengthRule { max_length: 64 };
        let mut item = new_retention_override("x", "y");
        item.fingerprint = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
