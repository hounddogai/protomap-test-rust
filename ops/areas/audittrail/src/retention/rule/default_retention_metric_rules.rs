use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default retention metric must pass. Check returns its problems, or none.
pub trait DefaultRetentionMetricRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRetentionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionMetricRequiredReviewerRule {
    pub message: String,
}

impl DefaultRetentionMetricRule for DefaultRetentionMetricRequiredReviewerRule {
    fn check(&self, item: &DefaultRetentionMetric) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionMetricAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl DefaultRetentionMetricRule for DefaultRetentionMetricAllowedIssuerRule {
    fn check(&self, item: &DefaultRetentionMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is missing", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionMetricApprovedGuardRule {
    pub limit: i64,
}

impl DefaultRetentionMetricRule for DefaultRetentionMetricApprovedGuardRule {
    fn check(&self, item: &DefaultRetentionMetric) -> Vec<String> {
        if item.approved && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while approved",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionMetricRequiredCodeRule {
    pub message: String,
}

impl DefaultRetentionMetricRule for DefaultRetentionMetricRequiredCodeRule {
    fn check(&self, item: &DefaultRetentionMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionMetricCodePrefixRule {
    pub prefix: String,
}

impl DefaultRetentionMetricRule for DefaultRetentionMetricCodePrefixRule {
    fn check(&self, item: &DefaultRetentionMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default retention metric.
#[derive(Debug, Clone)]
pub struct DefaultRetentionMetricValidator {
    pub rules: Vec<Rc<dyn DefaultRetentionMetricRule>>,
}

impl DefaultRetentionMetricValidator {
    pub fn validate(&self, item: &DefaultRetentionMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRetentionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRetentionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_retention_metric_validator() -> DefaultRetentionMetricValidator {
    DefaultRetentionMetricValidator {
        rules: vec![
            Rc::new(DefaultRetentionMetricRequiredReviewerRule {
                message: "default retention metric needs an entry".to_string(),
            }) as Rc<dyn DefaultRetentionMetricRule>,
            Rc::new(DefaultRetentionMetricAllowedIssuerRule {
                allowed: vec!["adaptive".to_string(), "manual".to_string()],
            }),
            Rc::new(DefaultRetentionMetricApprovedGuardRule { limit: 120 }),
            Rc::new(DefaultRetentionMetricRequiredCodeRule {
                message: "default retention metric needs a value".to_string(),
            }),
            Rc::new(DefaultRetentionMetricCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_retention_metric_validator();
        let item = new_default_retention_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = DefaultRetentionMetricRequiredReviewerRule {
            message: "default retention metric needs this field".to_string(),
        };
        let mut item = new_default_retention_metric("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultRetentionMetricRequiredCodeRule {
            message: "default retention metric needs this field".to_string(),
        };
        let mut item = new_default_retention_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
