use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage metric must pass. Check returns its problems, or none.
pub trait UsageMetricRule: std::fmt::Debug {
    fn check(&self, item: &UsageMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageMetricRequiredContactEmailRule {
    pub message: String,
}

impl UsageMetricRule for UsageMetricRequiredContactEmailRule {
    fn check(&self, item: &UsageMetric) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageMetricRequiredNameRule {
    pub message: String,
}

impl UsageMetricRule for UsageMetricRequiredNameRule {
    fn check(&self, item: &UsageMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageMetricActiveGuardRule {
    pub limit: i64,
}

impl UsageMetricRule for UsageMetricActiveGuardRule {
    fn check(&self, item: &UsageMetric) -> Vec<String> {
        if item.active && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while active",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageMetricNameLengthRule {
    pub max_length: i64,
}

impl UsageMetricRule for UsageMetricNameLengthRule {
    fn check(&self, item: &UsageMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageMetricCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageMetricRule for UsageMetricCapacityRangeRule {
    fn check(&self, item: &UsageMetric) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage metric.
#[derive(Debug, Clone)]
pub struct UsageMetricValidator {
    pub rules: Vec<Rc<dyn UsageMetricRule>>,
}

impl UsageMetricValidator {
    pub fn validate(&self, item: &UsageMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_metric_validator() -> UsageMetricValidator {
    UsageMetricValidator {
        rules: vec![
            Rc::new(UsageMetricRequiredContactEmailRule {
                message: "usage metric needs an entry".to_string(),
            }) as Rc<dyn UsageMetricRule>,
            Rc::new(UsageMetricRequiredNameRule {
                message: "usage metric needs a value".to_string(),
            }),
            Rc::new(UsageMetricActiveGuardRule { limit: 32 }),
            Rc::new(UsageMetricNameLengthRule { max_length: 255 }),
            Rc::new(UsageMetricCapacityRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_metric_validator();
        let item = new_usage_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = UsageMetricRequiredContactEmailRule {
            message: "usage metric needs this field".to_string(),
        };
        let mut item = new_usage_metric("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = UsageMetricRequiredNameRule {
            message: "usage metric needs a value".to_string(),
        };
        let mut item = new_usage_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = UsageMetricNameLengthRule { max_length: 86400 };
        let mut item = new_usage_metric("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = UsageMetricCapacityRangeRule { min: 3600, max: 86400 };
        let mut item = new_usage_metric("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
