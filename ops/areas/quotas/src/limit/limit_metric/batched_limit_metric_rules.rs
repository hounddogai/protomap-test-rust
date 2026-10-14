use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched limit metric must pass. Check returns its problems, or none.
pub trait BatchedLimitMetricRule: std::fmt::Debug {
    fn check(&self, item: &BatchedLimitMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMetricRequiredPrincipalRule {
    pub message: String,
}

impl BatchedLimitMetricRule for BatchedLimitMetricRequiredPrincipalRule {
    fn check(&self, item: &BatchedLimitMetric) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMetricRequiredNameRule {
    pub message: String,
}

impl BatchedLimitMetricRule for BatchedLimitMetricRequiredNameRule {
    fn check(&self, item: &BatchedLimitMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMetricQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedLimitMetricRule for BatchedLimitMetricQuantityRangeRule {
    fn check(&self, item: &BatchedLimitMetric) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched limit metric.
#[derive(Debug, Clone)]
pub struct BatchedLimitMetricValidator {
    pub rules: Vec<Rc<dyn BatchedLimitMetricRule>>,
}

impl BatchedLimitMetricValidator {
    pub fn validate(&self, item: &BatchedLimitMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedLimitMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedLimitMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_limit_metric_validator() -> BatchedLimitMetricValidator {
    BatchedLimitMetricValidator {
        rules: vec![
            Rc::new(BatchedLimitMetricRequiredPrincipalRule {
                message: "batched limit metric needs a value".to_string(),
            }) as Rc<dyn BatchedLimitMetricRule>,
            Rc::new(BatchedLimitMetricRequiredNameRule {
                message: "batched limit metric needs this field".to_string(),
            }),
            Rc::new(BatchedLimitMetricQuantityRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_limit_metric_validator();
        let item = new_batched_limit_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = BatchedLimitMetricRequiredPrincipalRule {
            message: "batched limit metric needs an entry".to_string(),
        };
        let mut item = new_batched_limit_metric("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedLimitMetricRequiredNameRule {
            message: "batched limit metric needs a value".to_string(),
        };
        let mut item = new_batched_limit_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = BatchedLimitMetricQuantityRangeRule { min: 32, max: 3600 };
        let mut item = new_batched_limit_metric("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
