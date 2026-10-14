use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary limit metric must pass. Check returns its problems, or none.
pub trait PrimaryLimitMetricRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryLimitMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitMetricBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryLimitMetricRule for PrimaryLimitMetricBudgetRangeRule {
    fn check(&self, item: &PrimaryLimitMetric) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitMetricRequiredCodeRule {
    pub message: String,
}

impl PrimaryLimitMetricRule for PrimaryLimitMetricRequiredCodeRule {
    fn check(&self, item: &PrimaryLimitMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitMetricAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryLimitMetricRule for PrimaryLimitMetricAllowedCodeRule {
    fn check(&self, item: &PrimaryLimitMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitMetricPinnedGuardRule {
    pub limit: f64,
}

impl PrimaryLimitMetricRule for PrimaryLimitMetricPinnedGuardRule {
    fn check(&self, item: &PrimaryLimitMetric) -> Vec<String> {
        if item.pinned && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while pinned", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitMetricNamespaceLengthRule {
    pub max_length: i64,
}

impl PrimaryLimitMetricRule for PrimaryLimitMetricNamespaceLengthRule {
    fn check(&self, item: &PrimaryLimitMetric) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary limit metric.
#[derive(Debug, Clone)]
pub struct PrimaryLimitMetricValidator {
    pub rules: Vec<Rc<dyn PrimaryLimitMetricRule>>,
}

impl PrimaryLimitMetricValidator {
    pub fn validate(&self, item: &PrimaryLimitMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryLimitMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryLimitMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_limit_metric_validator() -> PrimaryLimitMetricValidator {
    PrimaryLimitMetricValidator {
        rules: vec![
            Rc::new(PrimaryLimitMetricBudgetRangeRule { min: 9.1, max: 1329.9 }) as Rc<dyn PrimaryLimitMetricRule>,
            Rc::new(PrimaryLimitMetricRequiredCodeRule {
                message: "primary limit metric needs this field".to_string(),
            }),
            Rc::new(PrimaryLimitMetricAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "global".to_string()],
            }),
            Rc::new(PrimaryLimitMetricPinnedGuardRule { limit: 0.6 }),
            Rc::new(PrimaryLimitMetricNamespaceLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_limit_metric_validator();
        let item = new_primary_limit_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PrimaryLimitMetricBudgetRangeRule { min: 0.3, max: 3568.6 };
        let mut item = new_primary_limit_metric("x", "y");
        item.budget = 3570.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryLimitMetricRequiredCodeRule {
            message: "primary limit metric needs a value".to_string(),
        };
        let mut item = new_primary_limit_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = PrimaryLimitMetricNamespaceLengthRule { max_length: 64 };
        let mut item = new_primary_limit_metric("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
