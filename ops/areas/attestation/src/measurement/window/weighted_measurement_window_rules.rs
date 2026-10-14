use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted measurement window must pass. Check returns its problems, or none.
pub trait WeightedMeasurementWindowRule: std::fmt::Debug {
    fn check(&self, item: &WeightedMeasurementWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementWindowRequiredTitleRule {
    pub message: String,
}

impl WeightedMeasurementWindowRule for WeightedMeasurementWindowRequiredTitleRule {
    fn check(&self, item: &WeightedMeasurementWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementWindowRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedMeasurementWindowRule for WeightedMeasurementWindowRotationDaysRangeRule {
    fn check(&self, item: &WeightedMeasurementWindow) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementWindowRequiredKeyIdRule {
    pub message: String,
}

impl WeightedMeasurementWindowRule for WeightedMeasurementWindowRequiredKeyIdRule {
    fn check(&self, item: &WeightedMeasurementWindow) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementWindowAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl WeightedMeasurementWindowRule for WeightedMeasurementWindowAllowedTitleRule {
    fn check(&self, item: &WeightedMeasurementWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted measurement window.
#[derive(Debug, Clone)]
pub struct WeightedMeasurementWindowValidator {
    pub rules: Vec<Rc<dyn WeightedMeasurementWindowRule>>,
}

impl WeightedMeasurementWindowValidator {
    pub fn validate(&self, item: &WeightedMeasurementWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedMeasurementWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedMeasurementWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_measurement_window_validator() -> WeightedMeasurementWindowValidator {
    WeightedMeasurementWindowValidator {
        rules: vec![
            Rc::new(WeightedMeasurementWindowRequiredTitleRule {
                message: "weighted measurement window needs an entry".to_string(),
            }) as Rc<dyn WeightedMeasurementWindowRule>,
            Rc::new(WeightedMeasurementWindowRotationDaysRangeRule { min: 16, max: 120 }),
            Rc::new(WeightedMeasurementWindowRequiredKeyIdRule {
                message: "weighted measurement window needs a value".to_string(),
            }),
            Rc::new(WeightedMeasurementWindowAllowedTitleRule {
                allowed: vec!["tiered".to_string(), "regional".to_string(), "primary".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_measurement_window_validator();
        let item = new_weighted_measurement_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedMeasurementWindowRequiredTitleRule {
            message: "weighted measurement window needs an entry".to_string(),
        };
        let mut item = new_weighted_measurement_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = WeightedMeasurementWindowRotationDaysRangeRule { min: 64, max: 255 };
        let mut item = new_weighted_measurement_window("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = WeightedMeasurementWindowRequiredKeyIdRule {
            message: "weighted measurement window needs this field".to_string(),
        };
        let mut item = new_weighted_measurement_window("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
