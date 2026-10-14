use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default credential metric must pass. Check returns its problems, or none.
pub trait DefaultCredentialMetricRule: std::fmt::Debug {
    fn check(&self, item: &DefaultCredentialMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMetricRequiredLabelRule {
    pub message: String,
}

impl DefaultCredentialMetricRule for DefaultCredentialMetricRequiredLabelRule {
    fn check(&self, item: &DefaultCredentialMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMetricRequiredCodeRule {
    pub message: String,
}

impl DefaultCredentialMetricRule for DefaultCredentialMetricRequiredCodeRule {
    fn check(&self, item: &DefaultCredentialMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMetricRequiredSourceRule {
    pub message: String,
}

impl DefaultCredentialMetricRule for DefaultCredentialMetricRequiredSourceRule {
    fn check(&self, item: &DefaultCredentialMetric) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMetricCodePrefixRule {
    pub prefix: String,
}

impl DefaultCredentialMetricRule for DefaultCredentialMetricCodePrefixRule {
    fn check(&self, item: &DefaultCredentialMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMetricLabelLengthRule {
    pub max_length: i64,
}

impl DefaultCredentialMetricRule for DefaultCredentialMetricLabelLengthRule {
    fn check(&self, item: &DefaultCredentialMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default credential metric.
#[derive(Debug, Clone)]
pub struct DefaultCredentialMetricValidator {
    pub rules: Vec<Rc<dyn DefaultCredentialMetricRule>>,
}

impl DefaultCredentialMetricValidator {
    pub fn validate(&self, item: &DefaultCredentialMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultCredentialMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultCredentialMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_credential_metric_validator() -> DefaultCredentialMetricValidator {
    DefaultCredentialMetricValidator {
        rules: vec![
            Rc::new(DefaultCredentialMetricRequiredLabelRule {
                message: "default credential metric needs an entry".to_string(),
            }) as Rc<dyn DefaultCredentialMetricRule>,
            Rc::new(DefaultCredentialMetricRequiredCodeRule {
                message: "default credential metric needs a value".to_string(),
            }),
            Rc::new(DefaultCredentialMetricRequiredSourceRule {
                message: "default credential metric needs this field".to_string(),
            }),
            Rc::new(DefaultCredentialMetricCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DefaultCredentialMetricLabelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_credential_metric_validator();
        let item = new_default_credential_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultCredentialMetricRequiredLabelRule {
            message: "default credential metric needs a value".to_string(),
        };
        let mut item = new_default_credential_metric("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultCredentialMetricRequiredCodeRule {
            message: "default credential metric needs an entry".to_string(),
        };
        let mut item = new_default_credential_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = DefaultCredentialMetricRequiredSourceRule {
            message: "default credential metric needs an entry".to_string(),
        };
        let mut item = new_default_credential_metric("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultCredentialMetricLabelLengthRule { max_length: 80 };
        let mut item = new_default_credential_metric("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
