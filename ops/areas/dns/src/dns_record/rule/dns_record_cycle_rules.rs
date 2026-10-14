use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record cycle must pass. Check returns its problems, or none.
pub trait DnsRecordCycleRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCycleRequiredTitleRule {
    pub message: String,
}

impl DnsRecordCycleRule for DnsRecordCycleRequiredTitleRule {
    fn check(&self, item: &DnsRecordCycle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCycleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordCycleRule for DnsRecordCycleMinimumRangeRule {
    fn check(&self, item: &DnsRecordCycle) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCycleSummaryLengthRule {
    pub max_length: i64,
}

impl DnsRecordCycleRule for DnsRecordCycleSummaryLengthRule {
    fn check(&self, item: &DnsRecordCycle) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCycleBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordCycleRule for DnsRecordCycleBudgetRangeRule {
    fn check(&self, item: &DnsRecordCycle) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dns record cycle.
#[derive(Debug, Clone)]
pub struct DnsRecordCycleValidator {
    pub rules: Vec<Rc<dyn DnsRecordCycleRule>>,
}

impl DnsRecordCycleValidator {
    pub fn validate(&self, item: &DnsRecordCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_cycle_validator() -> DnsRecordCycleValidator {
    DnsRecordCycleValidator {
        rules: vec![
            Rc::new(DnsRecordCycleRequiredTitleRule {
                message: "dns record cycle needs this field".to_string(),
            }) as Rc<dyn DnsRecordCycleRule>,
            Rc::new(DnsRecordCycleMinimumRangeRule { min: 7.6, max: 68.7 }),
            Rc::new(DnsRecordCycleSummaryLengthRule { max_length: 64 }),
            Rc::new(DnsRecordCycleBudgetRangeRule { min: 3.4, max: 822.0 }),
        ],
    }
}
