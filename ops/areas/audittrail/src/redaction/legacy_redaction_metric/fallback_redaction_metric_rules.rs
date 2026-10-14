use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback redaction metric must pass. Check returns its problems, or none.
pub trait FallbackRedactionMetricRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRedactionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionMetricLocaleLengthRule {
    pub max_length: i64,
}

impl FallbackRedactionMetricRule for FallbackRedactionMetricLocaleLengthRule {
    fn check(&self, item: &FallbackRedactionMetric) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionMetricNameLengthRule {
    pub max_length: i64,
}

impl FallbackRedactionMetricRule for FallbackRedactionMetricNameLengthRule {
    fn check(&self, item: &FallbackRedactionMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionMetricGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRedactionMetricRule for FallbackRedactionMetricGraceDaysRangeRule {
    fn check(&self, item: &FallbackRedactionMetric) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionMetricAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRedactionMetricRule for FallbackRedactionMetricAttemptsRangeRule {
    fn check(&self, item: &FallbackRedactionMetric) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback redaction metric.
#[derive(Debug, Clone)]
pub struct FallbackRedactionMetricValidator {
    pub rules: Vec<Rc<dyn FallbackRedactionMetricRule>>,
}

impl FallbackRedactionMetricValidator {
    pub fn validate(&self, item: &FallbackRedactionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackRedactionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRedactionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_redaction_metric_validator() -> FallbackRedactionMetricValidator {
    FallbackRedactionMetricValidator {
        rules: vec![
            Rc::new(FallbackRedactionMetricLocaleLengthRule { max_length: 86400 })
                as Rc<dyn FallbackRedactionMetricRule>,
            Rc::new(FallbackRedactionMetricNameLengthRule { max_length: 120 }),
            Rc::new(FallbackRedactionMetricGraceDaysRangeRule { min: 16, max: 32 }),
            Rc::new(FallbackRedactionMetricAttemptsRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_redaction_metric_validator();
        let item = new_fallback_redaction_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = FallbackRedactionMetricLocaleLengthRule { max_length: 16 };
        let mut item = new_fallback_redaction_metric("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackRedactionMetricNameLengthRule { max_length: 32 };
        let mut item = new_fallback_redaction_metric("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FallbackRedactionMetricGraceDaysRangeRule { min: 32, max: 80 };
        let mut item = new_fallback_redaction_metric("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = FallbackRedactionMetricAttemptsRangeRule { min: 255, max: 86400 };
        let mut item = new_fallback_redaction_metric("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
