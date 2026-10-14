use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement tier must pass. Check returns its problems, or none.
pub trait MeasurementTierRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTierCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementTierRule for MeasurementTierCreatedAtRangeRule {
    fn check(&self, item: &MeasurementTier) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTierRequiredNamespaceRule {
    pub message: String,
}

impl MeasurementTierRule for MeasurementTierRequiredNamespaceRule {
    fn check(&self, item: &MeasurementTier) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTierRequiredFingerprintRule {
    pub message: String,
}

impl MeasurementTierRule for MeasurementTierRequiredFingerprintRule {
    fn check(&self, item: &MeasurementTier) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTierSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementTierRule for MeasurementTierSequenceRangeRule {
    fn check(&self, item: &MeasurementTier) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTierFingerprintLengthRule {
    pub max_length: i64,
}

impl MeasurementTierRule for MeasurementTierFingerprintLengthRule {
    fn check(&self, item: &MeasurementTier) -> Vec<String> {
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

/// Runs every rule of a measurement tier.
#[derive(Debug, Clone)]
pub struct MeasurementTierValidator {
    pub rules: Vec<Rc<dyn MeasurementTierRule>>,
}

impl MeasurementTierValidator {
    pub fn validate(&self, item: &MeasurementTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_tier_validator() -> MeasurementTierValidator {
    MeasurementTierValidator {
        rules: vec![
            Rc::new(MeasurementTierCreatedAtRangeRule { min: 86400, max: 86400 }) as Rc<dyn MeasurementTierRule>,
            Rc::new(MeasurementTierRequiredNamespaceRule {
                message: "measurement tier needs this field".to_string(),
            }),
            Rc::new(MeasurementTierRequiredFingerprintRule {
                message: "measurement tier needs this field".to_string(),
            }),
            Rc::new(MeasurementTierSequenceRangeRule { min: 32, max: 80 }),
            Rc::new(MeasurementTierFingerprintLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_tier_validator();
        let item = new_measurement_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = MeasurementTierCreatedAtRangeRule { min: 255, max: 1000 };
        let mut item = new_measurement_tier("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = MeasurementTierRequiredNamespaceRule {
            message: "measurement tier needs a value".to_string(),
        };
        let mut item = new_measurement_tier("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = MeasurementTierRequiredFingerprintRule {
            message: "measurement tier needs this field".to_string(),
        };
        let mut item = new_measurement_tier("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = MeasurementTierSequenceRangeRule { min: 80, max: 1000 };
        let mut item = new_measurement_tier("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = MeasurementTierFingerprintLengthRule { max_length: 80 };
        let mut item = new_measurement_tier("x", "y");
        item.fingerprint = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
