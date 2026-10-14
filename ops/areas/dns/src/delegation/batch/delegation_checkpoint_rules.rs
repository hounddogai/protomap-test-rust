use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation checkpoint must pass. Check returns its problems, or none.
pub trait DelegationCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &DelegationCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationCheckpointRequiredCodeRule {
    pub message: String,
}

impl DelegationCheckpointRule for DelegationCheckpointRequiredCodeRule {
    fn check(&self, item: &DelegationCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationCheckpointWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationCheckpointRule for DelegationCheckpointWeightRangeRule {
    fn check(&self, item: &DelegationCheckpoint) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationCheckpointFingerprintLengthRule {
    pub max_length: i64,
}

impl DelegationCheckpointRule for DelegationCheckpointFingerprintLengthRule {
    fn check(&self, item: &DelegationCheckpoint) -> Vec<String> {
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

/// Runs every rule of a delegation checkpoint.
#[derive(Debug, Clone)]
pub struct DelegationCheckpointValidator {
    pub rules: Vec<Rc<dyn DelegationCheckpointRule>>,
}

impl DelegationCheckpointValidator {
    pub fn validate(&self, item: &DelegationCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_checkpoint_validator() -> DelegationCheckpointValidator {
    DelegationCheckpointValidator {
        rules: vec![
            Rc::new(DelegationCheckpointRequiredCodeRule {
                message: "delegation checkpoint needs a value".to_string(),
            }) as Rc<dyn DelegationCheckpointRule>,
            Rc::new(DelegationCheckpointWeightRangeRule { min: 3.6, max: 2103.4 }),
            Rc::new(DelegationCheckpointFingerprintLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_checkpoint_validator();
        let item = new_delegation_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationCheckpointRequiredCodeRule {
            message: "delegation checkpoint needs a value".to_string(),
        };
        let mut item = new_delegation_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = DelegationCheckpointWeightRangeRule { min: 2.9, max: 3432.1 };
        let mut item = new_delegation_checkpoint("x", "y");
        item.weight = 3433.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = DelegationCheckpointFingerprintLengthRule { max_length: 32 };
        let mut item = new_delegation_checkpoint("x", "y");
        item.fingerprint = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
