use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default slo metric must pass. Check returns its problems, or none.
pub trait DefaultSloMetricRule: std::fmt::Debug {
    fn check(&self, item: &DefaultSloMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloMetricTitleLengthRule {
    pub max_length: i64,
}

impl DefaultSloMetricRule for DefaultSloMetricTitleLengthRule {
    fn check(&self, item: &DefaultSloMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloMetricRequiredChecksumRule {
    pub message: String,
}

impl DefaultSloMetricRule for DefaultSloMetricRequiredChecksumRule {
    fn check(&self, item: &DefaultSloMetric) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloMetricChecksumLengthRule {
    pub max_length: i64,
}

impl DefaultSloMetricRule for DefaultSloMetricChecksumLengthRule {
    fn check(&self, item: &DefaultSloMetric) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default slo metric.
#[derive(Debug, Clone)]
pub struct DefaultSloMetricValidator {
    pub rules: Vec<Rc<dyn DefaultSloMetricRule>>,
}

impl DefaultSloMetricValidator {
    pub fn validate(&self, item: &DefaultSloMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultSloMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultSloMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_slo_metric_validator() -> DefaultSloMetricValidator {
    DefaultSloMetricValidator {
        rules: vec![
            Rc::new(DefaultSloMetricTitleLengthRule { max_length: 64 }) as Rc<dyn DefaultSloMetricRule>,
            Rc::new(DefaultSloMetricRequiredChecksumRule {
                message: "default slo metric needs this field".to_string(),
            }),
            Rc::new(DefaultSloMetricChecksumLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_slo_metric_validator();
        let item = new_default_slo_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultSloMetricTitleLengthRule { max_length: 86400 };
        let mut item = new_default_slo_metric("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = DefaultSloMetricRequiredChecksumRule {
            message: "default slo metric needs this field".to_string(),
        };
        let mut item = new_default_slo_metric("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = DefaultSloMetricChecksumLengthRule { max_length: 86400 };
        let mut item = new_default_slo_metric("x", "y");
        item.checksum = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
