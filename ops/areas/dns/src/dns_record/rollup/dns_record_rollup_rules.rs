use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record rollup must pass. Check returns its problems, or none.
pub trait DnsRecordRollupRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRollupCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordRollupRule for DnsRecordRollupCeilingRangeRule {
    fn check(&self, item: &DnsRecordRollup) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRollupWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordRollupRule for DnsRecordRollupWindowDaysRangeRule {
    fn check(&self, item: &DnsRecordRollup) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRollupTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordRollupRule for DnsRecordRollupTtlSecondsRangeRule {
    fn check(&self, item: &DnsRecordRollup) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRollupApprovedGuardRule {
    pub limit: i64,
}

impl DnsRecordRollupRule for DnsRecordRollupApprovedGuardRule {
    fn check(&self, item: &DnsRecordRollup) -> Vec<String> {
        if item.approved && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while approved",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRollupRequiredCodeRule {
    pub message: String,
}

impl DnsRecordRollupRule for DnsRecordRollupRequiredCodeRule {
    fn check(&self, item: &DnsRecordRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record rollup.
#[derive(Debug, Clone)]
pub struct DnsRecordRollupValidator {
    pub rules: Vec<Rc<dyn DnsRecordRollupRule>>,
}

impl DnsRecordRollupValidator {
    pub fn validate(&self, item: &DnsRecordRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_rollup_validator() -> DnsRecordRollupValidator {
    DnsRecordRollupValidator {
        rules: vec![
            Rc::new(DnsRecordRollupCeilingRangeRule { min: 3.5, max: 2019.1 }) as Rc<dyn DnsRecordRollupRule>,
            Rc::new(DnsRecordRollupWindowDaysRangeRule { min: 80, max: 3600 }),
            Rc::new(DnsRecordRollupTtlSecondsRangeRule { min: 64, max: 1000 }),
            Rc::new(DnsRecordRollupApprovedGuardRule { limit: 80 }),
            Rc::new(DnsRecordRollupRequiredCodeRule {
                message: "dns record rollup needs a value".to_string(),
            }),
        ],
    }
}
