use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record variance must pass. Check returns its problems, or none.
pub trait DnsRecordVarianceRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordVarianceCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordVarianceRule for DnsRecordVarianceCostRangeRule {
    fn check(&self, item: &DnsRecordVariance) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordVarianceRequiredTitleRule {
    pub message: String,
}

impl DnsRecordVarianceRule for DnsRecordVarianceRequiredTitleRule {
    fn check(&self, item: &DnsRecordVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordVarianceCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordVarianceRule for DnsRecordVarianceCodePrefixRule {
    fn check(&self, item: &DnsRecordVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordVarianceRequiredCodeRule {
    pub message: String,
}

impl DnsRecordVarianceRule for DnsRecordVarianceRequiredCodeRule {
    fn check(&self, item: &DnsRecordVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record variance.
#[derive(Debug, Clone)]
pub struct DnsRecordVarianceValidator {
    pub rules: Vec<Rc<dyn DnsRecordVarianceRule>>,
}

impl DnsRecordVarianceValidator {
    pub fn validate(&self, item: &DnsRecordVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_variance_validator() -> DnsRecordVarianceValidator {
    DnsRecordVarianceValidator {
        rules: vec![
            Rc::new(DnsRecordVarianceCostRangeRule { min: 3.8, max: 4119.6 }) as Rc<dyn DnsRecordVarianceRule>,
            Rc::new(DnsRecordVarianceRequiredTitleRule {
                message: "dns record variance needs this field".to_string(),
            }),
            Rc::new(DnsRecordVarianceCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DnsRecordVarianceRequiredCodeRule {
                message: "dns record variance needs an entry".to_string(),
            }),
        ],
    }
}
