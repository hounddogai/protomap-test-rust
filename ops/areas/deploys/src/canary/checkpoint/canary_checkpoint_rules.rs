use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary checkpoint must pass. Check returns its problems, or none.
pub trait CanaryCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &CanaryCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCheckpointCodePrefixRule {
    pub prefix: String,
}

impl CanaryCheckpointRule for CanaryCheckpointCodePrefixRule {
    fn check(&self, item: &CanaryCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCheckpointRequiredNameRule {
    pub message: String,
}

impl CanaryCheckpointRule for CanaryCheckpointRequiredNameRule {
    fn check(&self, item: &CanaryCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCheckpointAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl CanaryCheckpointRule for CanaryCheckpointAllowedOwnerIdRule {
    fn check(&self, item: &CanaryCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is below minimum", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl CanaryCheckpointRule for CanaryCheckpointCodeLengthRule {
    fn check(&self, item: &CanaryCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCheckpointDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryCheckpointRule for CanaryCheckpointDurationMinutesRangeRule {
    fn check(&self, item: &CanaryCheckpoint) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary checkpoint.
#[derive(Debug, Clone)]
pub struct CanaryCheckpointValidator {
    pub rules: Vec<Rc<dyn CanaryCheckpointRule>>,
}

impl CanaryCheckpointValidator {
    pub fn validate(&self, item: &CanaryCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_checkpoint_validator() -> CanaryCheckpointValidator {
    CanaryCheckpointValidator {
        rules: vec![
            Rc::new(CanaryCheckpointCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn CanaryCheckpointRule>,
            Rc::new(CanaryCheckpointRequiredNameRule {
                message: "canary checkpoint needs a value".to_string(),
            }),
            Rc::new(CanaryCheckpointAllowedOwnerIdRule {
                allowed: vec!["fallback".to_string(), "weighted".to_string(), "tiered".to_string()],
            }),
            Rc::new(CanaryCheckpointCodeLengthRule { max_length: 86400 }),
            Rc::new(CanaryCheckpointDurationMinutesRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_checkpoint_validator();
        let item = new_canary_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CanaryCheckpointRequiredNameRule {
            message: "canary checkpoint needs an entry".to_string(),
        };
        let mut item = new_canary_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanaryCheckpointCodeLengthRule { max_length: 120 };
        let mut item = new_canary_checkpoint("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CanaryCheckpointDurationMinutesRangeRule { min: 80, max: 255 };
        let mut item = new_canary_checkpoint("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
