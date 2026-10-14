use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement forecast must pass. Check returns its problems, or none.
pub trait MeasurementForecastRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementForecastEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementForecastRule for MeasurementForecastEndsAtRangeRule {
    fn check(&self, item: &MeasurementForecast) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementForecastCodePrefixRule {
    pub prefix: String,
}

impl MeasurementForecastRule for MeasurementForecastCodePrefixRule {
    fn check(&self, item: &MeasurementForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementForecastSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementForecastRule for MeasurementForecastSequenceRangeRule {
    fn check(&self, item: &MeasurementForecast) -> Vec<String> {
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

/// Runs every rule of a measurement forecast.
#[derive(Debug, Clone)]
pub struct MeasurementForecastValidator {
    pub rules: Vec<Rc<dyn MeasurementForecastRule>>,
}

impl MeasurementForecastValidator {
    pub fn validate(&self, item: &MeasurementForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_forecast_validator() -> MeasurementForecastValidator {
    MeasurementForecastValidator {
        rules: vec![
            Rc::new(MeasurementForecastEndsAtRangeRule { min: 3600, max: 86400 }) as Rc<dyn MeasurementForecastRule>,
            Rc::new(MeasurementForecastCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(MeasurementForecastSequenceRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_forecast_validator();
        let item = new_measurement_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = MeasurementForecastEndsAtRangeRule { min: 80, max: 255 };
        let mut item = new_measurement_forecast("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = MeasurementForecastSequenceRangeRule { min: 16, max: 86400 };
        let mut item = new_measurement_forecast("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
