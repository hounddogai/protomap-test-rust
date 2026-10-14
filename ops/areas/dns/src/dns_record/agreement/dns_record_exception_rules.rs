use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record exception must pass. Check returns its problems, or none.
pub trait DnsRecordExceptionRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordExceptionExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordExceptionRule for DnsRecordExceptionExpiresAtRangeRule {
    fn check(&self, item: &DnsRecordException) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordExceptionAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DnsRecordExceptionRule for DnsRecordExceptionAllowedTitleRule {
    fn check(&self, item: &DnsRecordException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordExceptionMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordExceptionRule for DnsRecordExceptionMinimumRangeRule {
    fn check(&self, item: &DnsRecordException) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordExceptionRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordExceptionRule for DnsRecordExceptionRevisionRangeRule {
    fn check(&self, item: &DnsRecordException) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordExceptionRequiredChannelRule {
    pub message: String,
}

impl DnsRecordExceptionRule for DnsRecordExceptionRequiredChannelRule {
    fn check(&self, item: &DnsRecordException) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record exception.
#[derive(Debug, Clone)]
pub struct DnsRecordExceptionValidator {
    pub rules: Vec<Rc<dyn DnsRecordExceptionRule>>,
}

impl DnsRecordExceptionValidator {
    pub fn validate(&self, item: &DnsRecordException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_exception_validator() -> DnsRecordExceptionValidator {
    DnsRecordExceptionValidator {
        rules: vec![
            Rc::new(DnsRecordExceptionExpiresAtRangeRule { min: 64, max: 1000 }) as Rc<dyn DnsRecordExceptionRule>,
            Rc::new(DnsRecordExceptionAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(DnsRecordExceptionMinimumRangeRule { min: 7.7, max: 4495.9 }),
            Rc::new(DnsRecordExceptionRevisionRangeRule { min: 64, max: 255 }),
            Rc::new(DnsRecordExceptionRequiredChannelRule {
                message: "dns record exception needs a value".to_string(),
            }),
        ],
    }
}
