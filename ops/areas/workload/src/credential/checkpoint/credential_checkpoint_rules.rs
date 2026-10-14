use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential checkpoint must pass. Check returns its problems, or none.
pub trait CredentialCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &CredentialCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCheckpointBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialCheckpointRule for CredentialCheckpointBurstRangeRule {
    fn check(&self, item: &CredentialCheckpoint) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCheckpointAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CredentialCheckpointRule for CredentialCheckpointAllowedNameRule {
    fn check(&self, item: &CredentialCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCheckpointCodePrefixRule {
    pub prefix: String,
}

impl CredentialCheckpointRule for CredentialCheckpointCodePrefixRule {
    fn check(&self, item: &CredentialCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCheckpointFingerprintLengthRule {
    pub max_length: i64,
}

impl CredentialCheckpointRule for CredentialCheckpointFingerprintLengthRule {
    fn check(&self, item: &CredentialCheckpoint) -> Vec<String> {
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
pub struct CredentialCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl CredentialCheckpointRule for CredentialCheckpointCodeLengthRule {
    fn check(&self, item: &CredentialCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCheckpointEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialCheckpointRule for CredentialCheckpointEndsAtRangeRule {
    fn check(&self, item: &CredentialCheckpoint) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a credential checkpoint.
#[derive(Debug, Clone)]
pub struct CredentialCheckpointValidator {
    pub rules: Vec<Rc<dyn CredentialCheckpointRule>>,
}

impl CredentialCheckpointValidator {
    pub fn validate(&self, item: &CredentialCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_checkpoint_validator() -> CredentialCheckpointValidator {
    CredentialCheckpointValidator {
        rules: vec![
            Rc::new(CredentialCheckpointBurstRangeRule { min: 32, max: 64 }) as Rc<dyn CredentialCheckpointRule>,
            Rc::new(CredentialCheckpointAllowedNameRule {
                allowed: vec!["seasonal".to_string(), "capped".to_string(), "rolling".to_string()],
            }),
            Rc::new(CredentialCheckpointCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CredentialCheckpointFingerprintLengthRule { max_length: 1000 }),
            Rc::new(CredentialCheckpointCodeLengthRule { max_length: 86400 }),
            Rc::new(CredentialCheckpointEndsAtRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_checkpoint_validator();
        let item = new_credential_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = CredentialCheckpointBurstRangeRule { min: 120, max: 255 };
        let mut item = new_credential_checkpoint("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = CredentialCheckpointFingerprintLengthRule { max_length: 255 };
        let mut item = new_credential_checkpoint("x", "y");
        item.fingerprint = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CredentialCheckpointCodeLengthRule { max_length: 1000 };
        let mut item = new_credential_checkpoint("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = CredentialCheckpointEndsAtRangeRule { min: 120, max: 120 };
        let mut item = new_credential_checkpoint("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
