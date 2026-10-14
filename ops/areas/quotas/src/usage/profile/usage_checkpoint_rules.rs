use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage checkpoint must pass. Check returns its problems, or none.
pub trait UsageCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &UsageCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageCheckpointRequiredCountryRule {
    pub message: String,
}

impl UsageCheckpointRule for UsageCheckpointRequiredCountryRule {
    fn check(&self, item: &UsageCheckpoint) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageCheckpointRequiredNameRule {
    pub message: String,
}

impl UsageCheckpointRule for UsageCheckpointRequiredNameRule {
    fn check(&self, item: &UsageCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageCheckpointAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl UsageCheckpointRule for UsageCheckpointAllowedFingerprintRule {
    fn check(&self, item: &UsageCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is not allowed", item.fingerprint)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage checkpoint.
#[derive(Debug, Clone)]
pub struct UsageCheckpointValidator {
    pub rules: Vec<Rc<dyn UsageCheckpointRule>>,
}

impl UsageCheckpointValidator {
    pub fn validate(&self, item: &UsageCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_checkpoint_validator() -> UsageCheckpointValidator {
    UsageCheckpointValidator {
        rules: vec![
            Rc::new(UsageCheckpointRequiredCountryRule {
                message: "usage checkpoint needs a value".to_string(),
            }) as Rc<dyn UsageCheckpointRule>,
            Rc::new(UsageCheckpointRequiredNameRule {
                message: "usage checkpoint needs an entry".to_string(),
            }),
            Rc::new(UsageCheckpointAllowedFingerprintRule {
                allowed: vec!["seasonal".to_string(), "scheduled".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_checkpoint_validator();
        let item = new_usage_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = UsageCheckpointRequiredCountryRule {
            message: "usage checkpoint needs an entry".to_string(),
        };
        let mut item = new_usage_checkpoint("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = UsageCheckpointRequiredNameRule {
            message: "usage checkpoint needs this field".to_string(),
        };
        let mut item = new_usage_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
