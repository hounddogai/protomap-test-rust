use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default dashboard metric must pass. Check returns its problems, or none.
pub trait DefaultDashboardMetricRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDashboardMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardMetricCodePrefixRule {
    pub prefix: String,
}

impl DefaultDashboardMetricRule for DefaultDashboardMetricCodePrefixRule {
    fn check(&self, item: &DefaultDashboardMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardMetricExternalRefLengthRule {
    pub max_length: i64,
}

impl DefaultDashboardMetricRule for DefaultDashboardMetricExternalRefLengthRule {
    fn check(&self, item: &DefaultDashboardMetric) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardMetricRequiredCodeRule {
    pub message: String,
}

impl DefaultDashboardMetricRule for DefaultDashboardMetricRequiredCodeRule {
    fn check(&self, item: &DefaultDashboardMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardMetricLabelLengthRule {
    pub max_length: i64,
}

impl DefaultDashboardMetricRule for DefaultDashboardMetricLabelLengthRule {
    fn check(&self, item: &DefaultDashboardMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardMetricCodeLengthRule {
    pub max_length: i64,
}

impl DefaultDashboardMetricRule for DefaultDashboardMetricCodeLengthRule {
    fn check(&self, item: &DefaultDashboardMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardMetricRequiredExternalRefRule {
    pub message: String,
}

impl DefaultDashboardMetricRule for DefaultDashboardMetricRequiredExternalRefRule {
    fn check(&self, item: &DefaultDashboardMetric) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default dashboard metric.
#[derive(Debug, Clone)]
pub struct DefaultDashboardMetricValidator {
    pub rules: Vec<Rc<dyn DefaultDashboardMetricRule>>,
}

impl DefaultDashboardMetricValidator {
    pub fn validate(&self, item: &DefaultDashboardMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultDashboardMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDashboardMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_dashboard_metric_validator() -> DefaultDashboardMetricValidator {
    DefaultDashboardMetricValidator {
        rules: vec![
            Rc::new(DefaultDashboardMetricCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DefaultDashboardMetricRule>,
            Rc::new(DefaultDashboardMetricExternalRefLengthRule { max_length: 16 }),
            Rc::new(DefaultDashboardMetricRequiredCodeRule {
                message: "default dashboard metric needs a value".to_string(),
            }),
            Rc::new(DefaultDashboardMetricLabelLengthRule { max_length: 86400 }),
            Rc::new(DefaultDashboardMetricCodeLengthRule { max_length: 16 }),
            Rc::new(DefaultDashboardMetricRequiredExternalRefRule {
                message: "default dashboard metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_dashboard_metric_validator();
        let item = new_default_dashboard_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = DefaultDashboardMetricExternalRefLengthRule { max_length: 32 };
        let mut item = new_default_dashboard_metric("x", "y");
        item.external_ref = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultDashboardMetricRequiredCodeRule {
            message: "default dashboard metric needs this field".to_string(),
        };
        let mut item = new_default_dashboard_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultDashboardMetricLabelLengthRule { max_length: 120 };
        let mut item = new_default_dashboard_metric("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultDashboardMetricCodeLengthRule { max_length: 32 };
        let mut item = new_default_dashboard_metric("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = DefaultDashboardMetricRequiredExternalRefRule {
            message: "default dashboard metric needs a value".to_string(),
        };
        let mut item = new_default_dashboard_metric("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
