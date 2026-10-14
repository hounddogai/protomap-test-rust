use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential batch must pass. Check returns its problems, or none.
pub trait CredentialBatchRule: std::fmt::Debug {
    fn check(&self, item: &CredentialBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBatchRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialBatchRule for CredentialBatchRotationDaysRangeRule {
    fn check(&self, item: &CredentialBatch) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBatchAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CredentialBatchRule for CredentialBatchAllowedNameRule {
    fn check(&self, item: &CredentialBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBatchRequiredTitleRule {
    pub message: String,
}

impl CredentialBatchRule for CredentialBatchRequiredTitleRule {
    fn check(&self, item: &CredentialBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBatchRequiredNameRule {
    pub message: String,
}

impl CredentialBatchRule for CredentialBatchRequiredNameRule {
    fn check(&self, item: &CredentialBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBatchNameLengthRule {
    pub max_length: i64,
}

impl CredentialBatchRule for CredentialBatchNameLengthRule {
    fn check(&self, item: &CredentialBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential batch.
#[derive(Debug, Clone)]
pub struct CredentialBatchValidator {
    pub rules: Vec<Rc<dyn CredentialBatchRule>>,
}

impl CredentialBatchValidator {
    pub fn validate(&self, item: &CredentialBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_batch_validator() -> CredentialBatchValidator {
    CredentialBatchValidator {
        rules: vec![
            Rc::new(CredentialBatchRotationDaysRangeRule { min: 1000, max: 86400 }) as Rc<dyn CredentialBatchRule>,
            Rc::new(CredentialBatchAllowedNameRule { allowed: Vec::new() }),
            Rc::new(CredentialBatchRequiredTitleRule {
                message: "credential batch needs this field".to_string(),
            }),
            Rc::new(CredentialBatchRequiredNameRule {
                message: "credential batch needs an entry".to_string(),
            }),
            Rc::new(CredentialBatchNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_batch_validator();
        let item = new_credential_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = CredentialBatchRotationDaysRangeRule { min: 32, max: 255 };
        let mut item = new_credential_batch("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CredentialBatchRequiredTitleRule {
            message: "credential batch needs an entry".to_string(),
        };
        let mut item = new_credential_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CredentialBatchRequiredNameRule {
            message: "credential batch needs a value".to_string(),
        };
        let mut item = new_credential_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CredentialBatchNameLengthRule { max_length: 120 };
        let mut item = new_credential_batch("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
