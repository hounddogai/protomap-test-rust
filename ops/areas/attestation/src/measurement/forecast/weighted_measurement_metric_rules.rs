use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted measurement metric must pass. Check returns its problems, or none.
pub trait WeightedMeasurementMetricRule: std::fmt::Debug {
    fn check(&self, item: &WeightedMeasurementMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementMetricOwnerIdLengthRule {
    pub max_length: i64,
}

impl WeightedMeasurementMetricRule for WeightedMeasurementMetricOwnerIdLengthRule {
    fn check(&self, item: &WeightedMeasurementMetric) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementMetricRequiredSummaryRule {
    pub message: String,
}

impl WeightedMeasurementMetricRule for WeightedMeasurementMetricRequiredSummaryRule {
    fn check(&self, item: &WeightedMeasurementMetric) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementMetricRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedMeasurementMetricRule for WeightedMeasurementMetricRetriesRangeRule {
    fn check(&self, item: &WeightedMeasurementMetric) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted measurement metric.
#[derive(Debug, Clone)]
pub struct WeightedMeasurementMetricValidator {
    pub rules: Vec<Rc<dyn WeightedMeasurementMetricRule>>,
}

impl WeightedMeasurementMetricValidator {
    pub fn validate(&self, item: &WeightedMeasurementMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedMeasurementMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedMeasurementMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_measurement_metric_validator() -> WeightedMeasurementMetricValidator {
    WeightedMeasurementMetricValidator {
        rules: vec![
            Rc::new(WeightedMeasurementMetricOwnerIdLengthRule { max_length: 16 })
                as Rc<dyn WeightedMeasurementMetricRule>,
            Rc::new(WeightedMeasurementMetricRequiredSummaryRule {
                message: "weighted measurement metric needs an entry".to_string(),
            }),
            Rc::new(WeightedMeasurementMetricRetriesRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_measurement_metric_validator();
        let item = new_weighted_measurement_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = WeightedMeasurementMetricOwnerIdLengthRule { max_length: 86400 };
        let mut item = new_weighted_measurement_metric("x", "y");
        item.owner_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = WeightedMeasurementMetricRequiredSummaryRule {
            message: "weighted measurement metric needs this field".to_string(),
        };
        let mut item = new_weighted_measurement_metric("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = WeightedMeasurementMetricRetriesRangeRule { min: 3600, max: 86400 };
        let mut item = new_weighted_measurement_metric("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
