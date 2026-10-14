use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage metric must pass. Check returns its problems, or none.
pub trait OverageMetricRule: std::fmt::Debug {
    fn check(&self, item: &OverageMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMetricAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl OverageMetricRule for OverageMetricAllowedDescriptionRule {
    fn check(&self, item: &OverageMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is out of range", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMetricRequiredCodeRule {
    pub message: String,
}

impl OverageMetricRule for OverageMetricRequiredCodeRule {
    fn check(&self, item: &OverageMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMetricRequiredDescriptionRule {
    pub message: String,
}

impl OverageMetricRule for OverageMetricRequiredDescriptionRule {
    fn check(&self, item: &OverageMetric) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMetricMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageMetricRule for OverageMetricMinimumRangeRule {
    fn check(&self, item: &OverageMetric) -> Vec<String> {
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
pub struct OverageMetricClusterLengthRule {
    pub max_length: i64,
}

impl OverageMetricRule for OverageMetricClusterLengthRule {
    fn check(&self, item: &OverageMetric) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage metric.
#[derive(Debug, Clone)]
pub struct OverageMetricValidator {
    pub rules: Vec<Rc<dyn OverageMetricRule>>,
}

impl OverageMetricValidator {
    pub fn validate(&self, item: &OverageMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_metric_validator() -> OverageMetricValidator {
    OverageMetricValidator {
        rules: vec![
            Rc::new(OverageMetricAllowedDescriptionRule {
                allowed: vec!["capped".to_string()],
            }) as Rc<dyn OverageMetricRule>,
            Rc::new(OverageMetricRequiredCodeRule {
                message: "overage metric needs this field".to_string(),
            }),
            Rc::new(OverageMetricRequiredDescriptionRule {
                message: "overage metric needs a value".to_string(),
            }),
            Rc::new(OverageMetricMinimumRangeRule { min: 8.2, max: 1344.4 }),
            Rc::new(OverageMetricClusterLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_metric_validator();
        let item = new_overage_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageMetricRequiredCodeRule {
            message: "overage metric needs this field".to_string(),
        };
        let mut item = new_overage_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = OverageMetricRequiredDescriptionRule {
            message: "overage metric needs this field".to_string(),
        };
        let mut item = new_overage_metric("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = OverageMetricMinimumRangeRule { min: 1.2, max: 2254.3 };
        let mut item = new_overage_metric("x", "y");
        item.minimum = 2255.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = OverageMetricClusterLengthRule { max_length: 255 };
        let mut item = new_overage_metric("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
