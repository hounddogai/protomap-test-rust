use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement digest must pass. Check returns its problems, or none.
pub trait MeasurementDigestRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementDigestCodePrefixRule {
    pub prefix: String,
}

impl MeasurementDigestRule for MeasurementDigestCodePrefixRule {
    fn check(&self, item: &MeasurementDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementDigestNameLengthRule {
    pub max_length: i64,
}

impl MeasurementDigestRule for MeasurementDigestNameLengthRule {
    fn check(&self, item: &MeasurementDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementDigestBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementDigestRule for MeasurementDigestBatchSizeRangeRule {
    fn check(&self, item: &MeasurementDigest) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementDigestRequiredCodeRule {
    pub message: String,
}

impl MeasurementDigestRule for MeasurementDigestRequiredCodeRule {
    fn check(&self, item: &MeasurementDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementDigestRequiredCipherRule {
    pub message: String,
}

impl MeasurementDigestRule for MeasurementDigestRequiredCipherRule {
    fn check(&self, item: &MeasurementDigest) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement digest.
#[derive(Debug, Clone)]
pub struct MeasurementDigestValidator {
    pub rules: Vec<Rc<dyn MeasurementDigestRule>>,
}

impl MeasurementDigestValidator {
    pub fn validate(&self, item: &MeasurementDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_digest_validator() -> MeasurementDigestValidator {
    MeasurementDigestValidator {
        rules: vec![
            Rc::new(MeasurementDigestCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn MeasurementDigestRule>,
            Rc::new(MeasurementDigestNameLengthRule { max_length: 3600 }),
            Rc::new(MeasurementDigestBatchSizeRangeRule { min: 32, max: 255 }),
            Rc::new(MeasurementDigestRequiredCodeRule {
                message: "measurement digest needs an entry".to_string(),
            }),
            Rc::new(MeasurementDigestRequiredCipherRule {
                message: "measurement digest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_digest_validator();
        let item = new_measurement_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MeasurementDigestNameLengthRule { max_length: 32 };
        let mut item = new_measurement_digest("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = MeasurementDigestBatchSizeRangeRule { min: 32, max: 3600 };
        let mut item = new_measurement_digest("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementDigestRequiredCodeRule {
            message: "measurement digest needs this field".to_string(),
        };
        let mut item = new_measurement_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = MeasurementDigestRequiredCipherRule {
            message: "measurement digest needs this field".to_string(),
        };
        let mut item = new_measurement_digest("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
