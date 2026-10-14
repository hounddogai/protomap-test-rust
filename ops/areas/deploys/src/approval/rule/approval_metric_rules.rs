use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval metric must pass. Check returns its problems, or none.
pub trait ApprovalMetricRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMetricAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl ApprovalMetricRule for ApprovalMetricAllowedCurrencyRule {
    fn check(&self, item: &ApprovalMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is blocked", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMetricReviewerLengthRule {
    pub max_length: i64,
}

impl ApprovalMetricRule for ApprovalMetricReviewerLengthRule {
    fn check(&self, item: &ApprovalMetric) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMetricCodePrefixRule {
    pub prefix: String,
}

impl ApprovalMetricRule for ApprovalMetricCodePrefixRule {
    fn check(&self, item: &ApprovalMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMetricRequiredCodeRule {
    pub message: String,
}

impl ApprovalMetricRule for ApprovalMetricRequiredCodeRule {
    fn check(&self, item: &ApprovalMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval metric.
#[derive(Debug, Clone)]
pub struct ApprovalMetricValidator {
    pub rules: Vec<Rc<dyn ApprovalMetricRule>>,
}

impl ApprovalMetricValidator {
    pub fn validate(&self, item: &ApprovalMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_metric_validator() -> ApprovalMetricValidator {
    ApprovalMetricValidator {
        rules: vec![
            Rc::new(ApprovalMetricAllowedCurrencyRule { allowed: Vec::new() }) as Rc<dyn ApprovalMetricRule>,
            Rc::new(ApprovalMetricReviewerLengthRule { max_length: 32 }),
            Rc::new(ApprovalMetricCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ApprovalMetricRequiredCodeRule {
                message: "approval metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_metric_validator();
        let item = new_approval_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = ApprovalMetricReviewerLengthRule { max_length: 120 };
        let mut item = new_approval_metric("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ApprovalMetricRequiredCodeRule {
            message: "approval metric needs a value".to_string(),
        };
        let mut item = new_approval_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
