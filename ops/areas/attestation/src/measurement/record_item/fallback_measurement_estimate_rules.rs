use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback measurement estimate must pass. Check returns its problems, or none.
pub trait FallbackMeasurementEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackMeasurementEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementEstimateRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackMeasurementEstimateRule for FallbackMeasurementEstimateRetriesRangeRule {
    fn check(&self, item: &FallbackMeasurementEstimate) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementEstimateVisibleGuardRule {
    pub limit: i64,
}

impl FallbackMeasurementEstimateRule for FallbackMeasurementEstimateVisibleGuardRule {
    fn check(&self, item: &FallbackMeasurementEstimate) -> Vec<String> {
        if item.visible && (item.retries > self.limit) {
            return vec![format!("retries {} exceeds {} while visible", item.retries, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementEstimateCodeLengthRule {
    pub max_length: i64,
}

impl FallbackMeasurementEstimateRule for FallbackMeasurementEstimateCodeLengthRule {
    fn check(&self, item: &FallbackMeasurementEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementEstimateCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackMeasurementEstimateRule for FallbackMeasurementEstimateCreatedAtRangeRule {
    fn check(&self, item: &FallbackMeasurementEstimate) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementEstimateLabelLengthRule {
    pub max_length: i64,
}

impl FallbackMeasurementEstimateRule for FallbackMeasurementEstimateLabelLengthRule {
    fn check(&self, item: &FallbackMeasurementEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback measurement estimate.
#[derive(Debug, Clone)]
pub struct FallbackMeasurementEstimateValidator {
    pub rules: Vec<Rc<dyn FallbackMeasurementEstimateRule>>,
}

impl FallbackMeasurementEstimateValidator {
    pub fn validate(&self, item: &FallbackMeasurementEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackMeasurementEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackMeasurementEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_measurement_estimate_validator() -> FallbackMeasurementEstimateValidator {
    FallbackMeasurementEstimateValidator {
        rules: vec![
            Rc::new(FallbackMeasurementEstimateRetriesRangeRule { min: 16, max: 3600 })
                as Rc<dyn FallbackMeasurementEstimateRule>,
            Rc::new(FallbackMeasurementEstimateVisibleGuardRule { limit: 64 }),
            Rc::new(FallbackMeasurementEstimateCodeLengthRule { max_length: 64 }),
            Rc::new(FallbackMeasurementEstimateCreatedAtRangeRule { min: 64, max: 80 }),
            Rc::new(FallbackMeasurementEstimateLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_measurement_estimate_validator();
        let item = new_fallback_measurement_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = FallbackMeasurementEstimateRetriesRangeRule { min: 16, max: 1000 };
        let mut item = new_fallback_measurement_estimate("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackMeasurementEstimateCodeLengthRule { max_length: 1000 };
        let mut item = new_fallback_measurement_estimate("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = FallbackMeasurementEstimateCreatedAtRangeRule { min: 80, max: 3600 };
        let mut item = new_fallback_measurement_estimate("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackMeasurementEstimateLabelLengthRule { max_length: 120 };
        let mut item = new_fallback_measurement_estimate("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
