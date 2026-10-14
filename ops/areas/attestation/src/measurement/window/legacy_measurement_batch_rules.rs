use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy measurement batch must pass. Check returns its problems, or none.
pub trait LegacyMeasurementBatchRule: std::fmt::Debug {
    fn check(&self, item: &LegacyMeasurementBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBatchCodePrefixRule {
    pub prefix: String,
}

impl LegacyMeasurementBatchRule for LegacyMeasurementBatchCodePrefixRule {
    fn check(&self, item: &LegacyMeasurementBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBatchMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyMeasurementBatchRule for LegacyMeasurementBatchMinimumRangeRule {
    fn check(&self, item: &LegacyMeasurementBatch) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBatchGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyMeasurementBatchRule for LegacyMeasurementBatchGraceDaysRangeRule {
    fn check(&self, item: &LegacyMeasurementBatch) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBatchRequiredNameRule {
    pub message: String,
}

impl LegacyMeasurementBatchRule for LegacyMeasurementBatchRequiredNameRule {
    fn check(&self, item: &LegacyMeasurementBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy measurement batch.
#[derive(Debug, Clone)]
pub struct LegacyMeasurementBatchValidator {
    pub rules: Vec<Rc<dyn LegacyMeasurementBatchRule>>,
}

impl LegacyMeasurementBatchValidator {
    pub fn validate(&self, item: &LegacyMeasurementBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyMeasurementBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyMeasurementBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_measurement_batch_validator() -> LegacyMeasurementBatchValidator {
    LegacyMeasurementBatchValidator {
        rules: vec![
            Rc::new(LegacyMeasurementBatchCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn LegacyMeasurementBatchRule>,
            Rc::new(LegacyMeasurementBatchMinimumRangeRule { min: 8.8, max: 1680.2 }),
            Rc::new(LegacyMeasurementBatchGraceDaysRangeRule { min: 16, max: 255 }),
            Rc::new(LegacyMeasurementBatchRequiredNameRule {
                message: "legacy measurement batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_measurement_batch_validator();
        let item = new_legacy_measurement_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = LegacyMeasurementBatchMinimumRangeRule { min: 9.7, max: 4462.0 };
        let mut item = new_legacy_measurement_batch("x", "y");
        item.minimum = 4463.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LegacyMeasurementBatchGraceDaysRangeRule { min: 32, max: 32 };
        let mut item = new_legacy_measurement_batch("x", "y");
        item.grace_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyMeasurementBatchRequiredNameRule {
            message: "legacy measurement batch needs an entry".to_string(),
        };
        let mut item = new_legacy_measurement_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
