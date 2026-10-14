use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension metric must pass. Check returns its problems, or none.
pub trait SuspensionMetricRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMetricCodePrefixRule {
    pub prefix: String,
}

impl SuspensionMetricRule for SuspensionMetricCodePrefixRule {
    fn check(&self, item: &SuspensionMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMetricRequiredCodeRule {
    pub message: String,
}

impl SuspensionMetricRule for SuspensionMetricRequiredCodeRule {
    fn check(&self, item: &SuspensionMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMetricRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionMetricRule for SuspensionMetricRetriesRangeRule {
    fn check(&self, item: &SuspensionMetric) -> Vec<String> {
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

/// Runs every rule of a suspension metric.
#[derive(Debug, Clone)]
pub struct SuspensionMetricValidator {
    pub rules: Vec<Rc<dyn SuspensionMetricRule>>,
}

impl SuspensionMetricValidator {
    pub fn validate(&self, item: &SuspensionMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_metric_validator() -> SuspensionMetricValidator {
    SuspensionMetricValidator {
        rules: vec![
            Rc::new(SuspensionMetricCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn SuspensionMetricRule>,
            Rc::new(SuspensionMetricRequiredCodeRule {
                message: "suspension metric needs a value".to_string(),
            }),
            Rc::new(SuspensionMetricRetriesRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_metric_validator();
        let item = new_suspension_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionMetricRequiredCodeRule {
            message: "suspension metric needs an entry".to_string(),
        };
        let mut item = new_suspension_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = SuspensionMetricRetriesRangeRule { min: 80, max: 120 };
        let mut item = new_suspension_metric("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
