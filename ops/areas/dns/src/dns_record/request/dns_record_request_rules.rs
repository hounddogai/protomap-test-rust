use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record request must pass. Check returns its problems, or none.
pub trait DnsRecordRequestRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRequestChannelLengthRule {
    pub max_length: i64,
}

impl DnsRecordRequestRule for DnsRecordRequestChannelLengthRule {
    fn check(&self, item: &DnsRecordRequest) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRequestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordRequestRule for DnsRecordRequestDurationMinutesRangeRule {
    fn check(&self, item: &DnsRecordRequest) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRequestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordRequestRule for DnsRecordRequestAttemptsRangeRule {
    fn check(&self, item: &DnsRecordRequest) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRequestLabelLengthRule {
    pub max_length: i64,
}

impl DnsRecordRequestRule for DnsRecordRequestLabelLengthRule {
    fn check(&self, item: &DnsRecordRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record request.
#[derive(Debug, Clone)]
pub struct DnsRecordRequestValidator {
    pub rules: Vec<Rc<dyn DnsRecordRequestRule>>,
}

impl DnsRecordRequestValidator {
    pub fn validate(&self, item: &DnsRecordRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_request_validator() -> DnsRecordRequestValidator {
    DnsRecordRequestValidator {
        rules: vec![
            Rc::new(DnsRecordRequestChannelLengthRule { max_length: 32 }) as Rc<dyn DnsRecordRequestRule>,
            Rc::new(DnsRecordRequestDurationMinutesRangeRule { min: 64, max: 64 }),
            Rc::new(DnsRecordRequestAttemptsRangeRule { min: 32, max: 86400 }),
            Rc::new(DnsRecordRequestLabelLengthRule { max_length: 16 }),
        ],
    }
}
