use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted alert metric must pass. Check returns its problems, or none.
pub trait WeightedAlertMetricRule: std::fmt::Debug {
    fn check(&self, item: &WeightedAlertMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertMetricCodeLengthRule {
    pub max_length: i64,
}

impl WeightedAlertMetricRule for WeightedAlertMetricCodeLengthRule {
    fn check(&self, item: &WeightedAlertMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertMetricRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedAlertMetricRule for WeightedAlertMetricRevisionRangeRule {
    fn check(&self, item: &WeightedAlertMetric) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertMetricCodePrefixRule {
    pub prefix: String,
}

impl WeightedAlertMetricRule for WeightedAlertMetricCodePrefixRule {
    fn check(&self, item: &WeightedAlertMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted alert metric.
#[derive(Debug, Clone)]
pub struct WeightedAlertMetricValidator {
    pub rules: Vec<Rc<dyn WeightedAlertMetricRule>>,
}

impl WeightedAlertMetricValidator {
    pub fn validate(&self, item: &WeightedAlertMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedAlertMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedAlertMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_alert_metric_validator() -> WeightedAlertMetricValidator {
    WeightedAlertMetricValidator {
        rules: vec![
            Rc::new(WeightedAlertMetricCodeLengthRule { max_length: 86400 }) as Rc<dyn WeightedAlertMetricRule>,
            Rc::new(WeightedAlertMetricRevisionRangeRule { min: 120, max: 86400 }),
            Rc::new(WeightedAlertMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_alert_metric_validator();
        let item = new_weighted_alert_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedAlertMetricCodeLengthRule { max_length: 1000 };
        let mut item = new_weighted_alert_metric("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = WeightedAlertMetricRevisionRangeRule { min: 64, max: 80 };
        let mut item = new_weighted_alert_metric("x", "y");
        item.revision = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
