use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global dns record cycle must pass. Check returns its problems, or none.
pub trait GlobalDnsRecordCycleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalDnsRecordCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordCycleLabelLengthRule {
    pub max_length: i64,
}

impl GlobalDnsRecordCycleRule for GlobalDnsRecordCycleLabelLengthRule {
    fn check(&self, item: &GlobalDnsRecordCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordCycleRequiredSourceRule {
    pub message: String,
}

impl GlobalDnsRecordCycleRule for GlobalDnsRecordCycleRequiredSourceRule {
    fn check(&self, item: &GlobalDnsRecordCycle) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordCycleRequiredLabelRule {
    pub message: String,
}

impl GlobalDnsRecordCycleRule for GlobalDnsRecordCycleRequiredLabelRule {
    fn check(&self, item: &GlobalDnsRecordCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordCycleRequiredCodeRule {
    pub message: String,
}

impl GlobalDnsRecordCycleRule for GlobalDnsRecordCycleRequiredCodeRule {
    fn check(&self, item: &GlobalDnsRecordCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global dns record cycle.
#[derive(Debug, Clone)]
pub struct GlobalDnsRecordCycleValidator {
    pub rules: Vec<Rc<dyn GlobalDnsRecordCycleRule>>,
}

impl GlobalDnsRecordCycleValidator {
    pub fn validate(&self, item: &GlobalDnsRecordCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalDnsRecordCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalDnsRecordCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_dns_record_cycle_validator() -> GlobalDnsRecordCycleValidator {
    GlobalDnsRecordCycleValidator {
        rules: vec![
            Rc::new(GlobalDnsRecordCycleLabelLengthRule { max_length: 64 }) as Rc<dyn GlobalDnsRecordCycleRule>,
            Rc::new(GlobalDnsRecordCycleRequiredSourceRule {
                message: "global dns record cycle needs an entry".to_string(),
            }),
            Rc::new(GlobalDnsRecordCycleRequiredLabelRule {
                message: "global dns record cycle needs a value".to_string(),
            }),
            Rc::new(GlobalDnsRecordCycleRequiredCodeRule {
                message: "global dns record cycle needs this field".to_string(),
            }),
        ],
    }
}
