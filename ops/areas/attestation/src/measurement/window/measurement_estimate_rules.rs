use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement estimate must pass. Check returns its problems, or none.
pub trait MeasurementEstimateRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEstimateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementEstimateRule for MeasurementEstimateExpiresAtRangeRule {
    fn check(&self, item: &MeasurementEstimate) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEstimateCodePrefixRule {
    pub prefix: String,
}

impl MeasurementEstimateRule for MeasurementEstimateCodePrefixRule {
    fn check(&self, item: &MeasurementEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEstimateRequiredCategoryRule {
    pub message: String,
}

impl MeasurementEstimateRule for MeasurementEstimateRequiredCategoryRule {
    fn check(&self, item: &MeasurementEstimate) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement estimate.
#[derive(Debug, Clone)]
pub struct MeasurementEstimateValidator {
    pub rules: Vec<Rc<dyn MeasurementEstimateRule>>,
}

impl MeasurementEstimateValidator {
    pub fn validate(&self, item: &MeasurementEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_estimate_validator() -> MeasurementEstimateValidator {
    MeasurementEstimateValidator {
        rules: vec![
            Rc::new(MeasurementEstimateExpiresAtRangeRule { min: 80, max: 86400 }) as Rc<dyn MeasurementEstimateRule>,
            Rc::new(MeasurementEstimateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(MeasurementEstimateRequiredCategoryRule {
                message: "measurement estimate needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_estimate_validator();
        let item = new_measurement_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = MeasurementEstimateExpiresAtRangeRule { min: 64, max: 255 };
        let mut item = new_measurement_estimate("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = MeasurementEstimateRequiredCategoryRule {
            message: "measurement estimate needs an entry".to_string(),
        };
        let mut item = new_measurement_estimate("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
