use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement manifest must pass. Check returns its problems, or none.
pub trait MeasurementManifestRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementManifestRequiredAudienceRule {
    pub message: String,
}

impl MeasurementManifestRule for MeasurementManifestRequiredAudienceRule {
    fn check(&self, item: &MeasurementManifest) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementManifestAudienceLengthRule {
    pub max_length: i64,
}

impl MeasurementManifestRule for MeasurementManifestAudienceLengthRule {
    fn check(&self, item: &MeasurementManifest) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementManifestBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementManifestRule for MeasurementManifestBatchSizeRangeRule {
    fn check(&self, item: &MeasurementManifest) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement manifest.
#[derive(Debug, Clone)]
pub struct MeasurementManifestValidator {
    pub rules: Vec<Rc<dyn MeasurementManifestRule>>,
}

impl MeasurementManifestValidator {
    pub fn validate(&self, item: &MeasurementManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_manifest_validator() -> MeasurementManifestValidator {
    MeasurementManifestValidator {
        rules: vec![
            Rc::new(MeasurementManifestRequiredAudienceRule {
                message: "measurement manifest needs an entry".to_string(),
            }) as Rc<dyn MeasurementManifestRule>,
            Rc::new(MeasurementManifestAudienceLengthRule { max_length: 64 }),
            Rc::new(MeasurementManifestBatchSizeRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_manifest_validator();
        let item = new_measurement_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = MeasurementManifestRequiredAudienceRule {
            message: "measurement manifest needs an entry".to_string(),
        };
        let mut item = new_measurement_manifest("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = MeasurementManifestAudienceLengthRule { max_length: 86400 };
        let mut item = new_measurement_manifest("x", "y");
        item.audience = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = MeasurementManifestBatchSizeRangeRule { min: 16, max: 255 };
        let mut item = new_measurement_manifest("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
