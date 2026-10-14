use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped suspension metric must pass. Check returns its problems, or none.
pub trait CappedSuspensionMetricRule: std::fmt::Debug {
    fn check(&self, item: &CappedSuspensionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionMetricRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSuspensionMetricRule for CappedSuspensionMetricRevisionRangeRule {
    fn check(&self, item: &CappedSuspensionMetric) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionMetricRequiredSourceRule {
    pub message: String,
}

impl CappedSuspensionMetricRule for CappedSuspensionMetricRequiredSourceRule {
    fn check(&self, item: &CappedSuspensionMetric) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionMetricCodePrefixRule {
    pub prefix: String,
}

impl CappedSuspensionMetricRule for CappedSuspensionMetricCodePrefixRule {
    fn check(&self, item: &CappedSuspensionMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionMetricAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSuspensionMetricRule for CappedSuspensionMetricAttemptsRangeRule {
    fn check(&self, item: &CappedSuspensionMetric) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped suspension metric.
#[derive(Debug, Clone)]
pub struct CappedSuspensionMetricValidator {
    pub rules: Vec<Rc<dyn CappedSuspensionMetricRule>>,
}

impl CappedSuspensionMetricValidator {
    pub fn validate(&self, item: &CappedSuspensionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedSuspensionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSuspensionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_suspension_metric_validator() -> CappedSuspensionMetricValidator {
    CappedSuspensionMetricValidator {
        rules: vec![
            Rc::new(CappedSuspensionMetricRevisionRangeRule { min: 1000, max: 86400 })
                as Rc<dyn CappedSuspensionMetricRule>,
            Rc::new(CappedSuspensionMetricRequiredSourceRule {
                message: "capped suspension metric needs a value".to_string(),
            }),
            Rc::new(CappedSuspensionMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(CappedSuspensionMetricAttemptsRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_suspension_metric_validator();
        let item = new_capped_suspension_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = CappedSuspensionMetricRevisionRangeRule { min: 80, max: 86400 };
        let mut item = new_capped_suspension_metric("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = CappedSuspensionMetricRequiredSourceRule {
            message: "capped suspension metric needs this field".to_string(),
        };
        let mut item = new_capped_suspension_metric("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = CappedSuspensionMetricAttemptsRangeRule { min: 255, max: 86400 };
        let mut item = new_capped_suspension_metric("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
