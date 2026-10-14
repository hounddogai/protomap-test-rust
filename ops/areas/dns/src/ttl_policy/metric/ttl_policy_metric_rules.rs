use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy metric must pass. Check returns its problems, or none.
pub trait TtlPolicyMetricRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMetricRequiredContactEmailRule {
    pub message: String,
}

impl TtlPolicyMetricRule for TtlPolicyMetricRequiredContactEmailRule {
    fn check(&self, item: &TtlPolicyMetric) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMetricContactEmailLengthRule {
    pub max_length: i64,
}

impl TtlPolicyMetricRule for TtlPolicyMetricContactEmailLengthRule {
    fn check(&self, item: &TtlPolicyMetric) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMetricCodeLengthRule {
    pub max_length: i64,
}

impl TtlPolicyMetricRule for TtlPolicyMetricCodeLengthRule {
    fn check(&self, item: &TtlPolicyMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMetricNameLengthRule {
    pub max_length: i64,
}

impl TtlPolicyMetricRule for TtlPolicyMetricNameLengthRule {
    fn check(&self, item: &TtlPolicyMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy metric.
#[derive(Debug, Clone)]
pub struct TtlPolicyMetricValidator {
    pub rules: Vec<Rc<dyn TtlPolicyMetricRule>>,
}

impl TtlPolicyMetricValidator {
    pub fn validate(&self, item: &TtlPolicyMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_metric_validator() -> TtlPolicyMetricValidator {
    TtlPolicyMetricValidator {
        rules: vec![
            Rc::new(TtlPolicyMetricRequiredContactEmailRule {
                message: "ttl policy metric needs this field".to_string(),
            }) as Rc<dyn TtlPolicyMetricRule>,
            Rc::new(TtlPolicyMetricContactEmailLengthRule { max_length: 16 }),
            Rc::new(TtlPolicyMetricCodeLengthRule { max_length: 120 }),
            Rc::new(TtlPolicyMetricNameLengthRule { max_length: 255 }),
        ],
    }
}
