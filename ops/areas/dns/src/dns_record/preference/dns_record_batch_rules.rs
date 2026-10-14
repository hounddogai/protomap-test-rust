use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record batch must pass. Check returns its problems, or none.
pub trait DnsRecordBatchRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBatchAllowedHostRule {
    pub allowed: Vec<String>,
}

impl DnsRecordBatchRule for DnsRecordBatchAllowedHostRule {
    fn check(&self, item: &DnsRecordBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is above maximum", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBatchRequiredIssuerRule {
    pub message: String,
}

impl DnsRecordBatchRule for DnsRecordBatchRequiredIssuerRule {
    fn check(&self, item: &DnsRecordBatch) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBatchRequiredCodeRule {
    pub message: String,
}

impl DnsRecordBatchRule for DnsRecordBatchRequiredCodeRule {
    fn check(&self, item: &DnsRecordBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record batch.
#[derive(Debug, Clone)]
pub struct DnsRecordBatchValidator {
    pub rules: Vec<Rc<dyn DnsRecordBatchRule>>,
}

impl DnsRecordBatchValidator {
    pub fn validate(&self, item: &DnsRecordBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_batch_validator() -> DnsRecordBatchValidator {
    DnsRecordBatchValidator {
        rules: vec![
            Rc::new(DnsRecordBatchAllowedHostRule {
                allowed: vec!["scheduled".to_string(), "pending".to_string(), "capped".to_string()],
            }) as Rc<dyn DnsRecordBatchRule>,
            Rc::new(DnsRecordBatchRequiredIssuerRule {
                message: "dns record batch needs this field".to_string(),
            }),
            Rc::new(DnsRecordBatchRequiredCodeRule {
                message: "dns record batch needs this field".to_string(),
            }),
        ],
    }
}
