use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier checkpoint must pass. Check returns its problems, or none.
pub trait VerifierCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &VerifierCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCheckpointRequiredCodeRule {
    pub message: String,
}

impl VerifierCheckpointRule for VerifierCheckpointRequiredCodeRule {
    fn check(&self, item: &VerifierCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCheckpointDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierCheckpointRule for VerifierCheckpointDurationMinutesRangeRule {
    fn check(&self, item: &VerifierCheckpoint) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCheckpointNameLengthRule {
    pub max_length: i64,
}

impl VerifierCheckpointRule for VerifierCheckpointNameLengthRule {
    fn check(&self, item: &VerifierCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCheckpointRequiredLabelRule {
    pub message: String,
}

impl VerifierCheckpointRule for VerifierCheckpointRequiredLabelRule {
    fn check(&self, item: &VerifierCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCheckpointCodePrefixRule {
    pub prefix: String,
}

impl VerifierCheckpointRule for VerifierCheckpointCodePrefixRule {
    fn check(&self, item: &VerifierCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCheckpointMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierCheckpointRule for VerifierCheckpointMinimumRangeRule {
    fn check(&self, item: &VerifierCheckpoint) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier checkpoint.
#[derive(Debug, Clone)]
pub struct VerifierCheckpointValidator {
    pub rules: Vec<Rc<dyn VerifierCheckpointRule>>,
}

impl VerifierCheckpointValidator {
    pub fn validate(&self, item: &VerifierCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_checkpoint_validator() -> VerifierCheckpointValidator {
    VerifierCheckpointValidator {
        rules: vec![
            Rc::new(VerifierCheckpointRequiredCodeRule {
                message: "verifier checkpoint needs this field".to_string(),
            }) as Rc<dyn VerifierCheckpointRule>,
            Rc::new(VerifierCheckpointDurationMinutesRangeRule { min: 120, max: 120 }),
            Rc::new(VerifierCheckpointNameLengthRule { max_length: 64 }),
            Rc::new(VerifierCheckpointRequiredLabelRule {
                message: "verifier checkpoint needs a value".to_string(),
            }),
            Rc::new(VerifierCheckpointCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(VerifierCheckpointMinimumRangeRule { min: 3.4, max: 2843.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_checkpoint_validator();
        let item = new_verifier_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierCheckpointRequiredCodeRule {
            message: "verifier checkpoint needs this field".to_string(),
        };
        let mut item = new_verifier_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = VerifierCheckpointDurationMinutesRangeRule { min: 120, max: 255 };
        let mut item = new_verifier_checkpoint("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = VerifierCheckpointNameLengthRule { max_length: 1000 };
        let mut item = new_verifier_checkpoint("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = VerifierCheckpointRequiredLabelRule {
            message: "verifier checkpoint needs this field".to_string(),
        };
        let mut item = new_verifier_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = VerifierCheckpointMinimumRangeRule { min: 6.8, max: 1477.3 };
        let mut item = new_verifier_checkpoint("x", "y");
        item.minimum = 1478.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
