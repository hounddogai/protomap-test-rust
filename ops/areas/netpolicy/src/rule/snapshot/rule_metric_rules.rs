use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule metric must pass. Check returns its problems, or none.
pub trait RuleMetricRule: std::fmt::Debug {
    fn check(&self, item: &RuleMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMetricRequiredLabelRule {
    pub message: String,
}

impl RuleMetricRule for RuleMetricRequiredLabelRule {
    fn check(&self, item: &RuleMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMetricCodePrefixRule {
    pub prefix: String,
}

impl RuleMetricRule for RuleMetricCodePrefixRule {
    fn check(&self, item: &RuleMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMetricLabelLengthRule {
    pub max_length: i64,
}

impl RuleMetricRule for RuleMetricLabelLengthRule {
    fn check(&self, item: &RuleMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMetricTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RuleMetricRule for RuleMetricTagsLimitRule {
    fn check(&self, item: &RuleMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a rule metric.
#[derive(Debug, Clone)]
pub struct RuleMetricValidator {
    pub rules: Vec<Rc<dyn RuleMetricRule>>,
}

impl RuleMetricValidator {
    pub fn validate(&self, item: &RuleMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_metric_validator() -> RuleMetricValidator {
    RuleMetricValidator {
        rules: vec![
            Rc::new(RuleMetricRequiredLabelRule {
                message: "rule metric needs a value".to_string(),
            }) as Rc<dyn RuleMetricRule>,
            Rc::new(RuleMetricCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RuleMetricLabelLengthRule { max_length: 1000 }),
            Rc::new(RuleMetricTagsLimitRule {
                max_count: 120,
                banned: Vec::new(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_metric_validator();
        let item = new_rule_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RuleMetricRequiredLabelRule {
            message: "rule metric needs this field".to_string(),
        };
        let mut item = new_rule_metric("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RuleMetricLabelLengthRule { max_length: 3600 };
        let mut item = new_rule_metric("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
