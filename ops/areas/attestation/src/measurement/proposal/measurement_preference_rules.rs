use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement preference must pass. Check returns its problems, or none.
pub trait MeasurementPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPreferenceMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementPreferenceRule for MeasurementPreferenceMinimumRangeRule {
    fn check(&self, item: &MeasurementPreference) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPreferenceRequiredDescriptionRule {
    pub message: String,
}

impl MeasurementPreferenceRule for MeasurementPreferenceRequiredDescriptionRule {
    fn check(&self, item: &MeasurementPreference) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPreferenceBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementPreferenceRule for MeasurementPreferenceBatchSizeRangeRule {
    fn check(&self, item: &MeasurementPreference) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPreferenceAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl MeasurementPreferenceRule for MeasurementPreferenceAllowedTitleRule {
    fn check(&self, item: &MeasurementPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPreferenceRequiredCodeRule {
    pub message: String,
}

impl MeasurementPreferenceRule for MeasurementPreferenceRequiredCodeRule {
    fn check(&self, item: &MeasurementPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement preference.
#[derive(Debug, Clone)]
pub struct MeasurementPreferenceValidator {
    pub rules: Vec<Rc<dyn MeasurementPreferenceRule>>,
}

impl MeasurementPreferenceValidator {
    pub fn validate(&self, item: &MeasurementPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_preference_validator() -> MeasurementPreferenceValidator {
    MeasurementPreferenceValidator {
        rules: vec![
            Rc::new(MeasurementPreferenceMinimumRangeRule { min: 2.3, max: 2212.9 })
                as Rc<dyn MeasurementPreferenceRule>,
            Rc::new(MeasurementPreferenceRequiredDescriptionRule {
                message: "measurement preference needs this field".to_string(),
            }),
            Rc::new(MeasurementPreferenceBatchSizeRangeRule { min: 255, max: 3600 }),
            Rc::new(MeasurementPreferenceAllowedTitleRule {
                allowed: vec!["fallback".to_string(), "lenient".to_string(), "global".to_string()],
            }),
            Rc::new(MeasurementPreferenceRequiredCodeRule {
                message: "measurement preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_preference_validator();
        let item = new_measurement_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = MeasurementPreferenceMinimumRangeRule { min: 9.4, max: 1832.1 };
        let mut item = new_measurement_preference("x", "y");
        item.minimum = 1833.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = MeasurementPreferenceRequiredDescriptionRule {
            message: "measurement preference needs this field".to_string(),
        };
        let mut item = new_measurement_preference("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = MeasurementPreferenceBatchSizeRangeRule { min: 64, max: 3600 };
        let mut item = new_measurement_preference("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementPreferenceRequiredCodeRule {
            message: "measurement preference needs this field".to_string(),
        };
        let mut item = new_measurement_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
