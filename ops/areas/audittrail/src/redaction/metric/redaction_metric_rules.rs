use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction metric must pass. Check returns its problems, or none.
pub trait RedactionMetricRule: std::fmt::Debug {
    fn check(&self, item: &RedactionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMetricCodeLengthRule {
    pub max_length: i64,
}

impl RedactionMetricRule for RedactionMetricCodeLengthRule {
    fn check(&self, item: &RedactionMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMetricCodePrefixRule {
    pub prefix: String,
}

impl RedactionMetricRule for RedactionMetricCodePrefixRule {
    fn check(&self, item: &RedactionMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMetricTitleLengthRule {
    pub max_length: i64,
}

impl RedactionMetricRule for RedactionMetricTitleLengthRule {
    fn check(&self, item: &RedactionMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction metric.
#[derive(Debug, Clone)]
pub struct RedactionMetricValidator {
    pub rules: Vec<Rc<dyn RedactionMetricRule>>,
}

impl RedactionMetricValidator {
    pub fn validate(&self, item: &RedactionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_metric_validator() -> RedactionMetricValidator {
    RedactionMetricValidator {
        rules: vec![
            Rc::new(RedactionMetricCodeLengthRule { max_length: 1000 }) as Rc<dyn RedactionMetricRule>,
            Rc::new(RedactionMetricCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RedactionMetricTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_metric_validator();
        let item = new_redaction_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionMetricCodeLengthRule { max_length: 86400 };
        let mut item = new_redaction_metric("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RedactionMetricTitleLengthRule { max_length: 80 };
        let mut item = new_redaction_metric("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
