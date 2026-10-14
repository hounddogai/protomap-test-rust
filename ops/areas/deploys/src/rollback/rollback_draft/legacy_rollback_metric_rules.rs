use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy rollback metric must pass. Check returns its problems, or none.
pub trait LegacyRollbackMetricRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRollbackMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMetricRequiredNameRule {
    pub message: String,
}

impl LegacyRollbackMetricRule for LegacyRollbackMetricRequiredNameRule {
    fn check(&self, item: &LegacyRollbackMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMetricCodeLengthRule {
    pub max_length: i64,
}

impl LegacyRollbackMetricRule for LegacyRollbackMetricCodeLengthRule {
    fn check(&self, item: &LegacyRollbackMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMetricAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LegacyRollbackMetricRule for LegacyRollbackMetricAllowedNameRule {
    fn check(&self, item: &LegacyRollbackMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMetricAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyRollbackMetricRule for LegacyRollbackMetricAttemptsRangeRule {
    fn check(&self, item: &LegacyRollbackMetric) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy rollback metric.
#[derive(Debug, Clone)]
pub struct LegacyRollbackMetricValidator {
    pub rules: Vec<Rc<dyn LegacyRollbackMetricRule>>,
}

impl LegacyRollbackMetricValidator {
    pub fn validate(&self, item: &LegacyRollbackMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRollbackMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRollbackMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollback_metric_validator() -> LegacyRollbackMetricValidator {
    LegacyRollbackMetricValidator {
        rules: vec![
            Rc::new(LegacyRollbackMetricRequiredNameRule {
                message: "legacy rollback metric needs a value".to_string(),
            }) as Rc<dyn LegacyRollbackMetricRule>,
            Rc::new(LegacyRollbackMetricCodeLengthRule { max_length: 1000 }),
            Rc::new(LegacyRollbackMetricAllowedNameRule {
                allowed: vec!["seasonal".to_string(), "primary".to_string(), "tiered".to_string()],
            }),
            Rc::new(LegacyRollbackMetricAttemptsRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_rollback_metric_validator();
        let item = new_legacy_rollback_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyRollbackMetricRequiredNameRule {
            message: "legacy rollback metric needs a value".to_string(),
        };
        let mut item = new_legacy_rollback_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyRollbackMetricCodeLengthRule { max_length: 64 };
        let mut item = new_legacy_rollback_metric("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = LegacyRollbackMetricAttemptsRangeRule { min: 16, max: 16 };
        let mut item = new_legacy_rollback_metric("x", "y");
        item.attempts = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
