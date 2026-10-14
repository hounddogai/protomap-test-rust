use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone quota must pass. Check returns its problems, or none.
pub trait ZoneQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ZoneQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneQuotaCodeLengthRule {
    pub max_length: i64,
}

impl ZoneQuotaRule for ZoneQuotaCodeLengthRule {
    fn check(&self, item: &ZoneQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneQuotaAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl ZoneQuotaRule for ZoneQuotaAllowedIssuerRule {
    fn check(&self, item: &ZoneQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is duplicated", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneQuotaCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneQuotaRule for ZoneQuotaCreatedAtRangeRule {
    fn check(&self, item: &ZoneQuota) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneQuotaLabelLengthRule {
    pub max_length: i64,
}

impl ZoneQuotaRule for ZoneQuotaLabelLengthRule {
    fn check(&self, item: &ZoneQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneQuotaChannelLengthRule {
    pub max_length: i64,
}

impl ZoneQuotaRule for ZoneQuotaChannelLengthRule {
    fn check(&self, item: &ZoneQuota) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneQuotaCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneQuotaRule for ZoneQuotaCeilingRangeRule {
    fn check(&self, item: &ZoneQuota) -> Vec<String> {
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

/// Runs every rule of a zone quota.
#[derive(Debug, Clone)]
pub struct ZoneQuotaValidator {
    pub rules: Vec<Rc<dyn ZoneQuotaRule>>,
}

impl ZoneQuotaValidator {
    pub fn validate(&self, item: &ZoneQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_quota_validator() -> ZoneQuotaValidator {
    ZoneQuotaValidator {
        rules: vec![
            Rc::new(ZoneQuotaCodeLengthRule { max_length: 120 }) as Rc<dyn ZoneQuotaRule>,
            Rc::new(ZoneQuotaAllowedIssuerRule {
                allowed: vec!["tiered".to_string(), "fallback".to_string(), "rolling".to_string()],
            }),
            Rc::new(ZoneQuotaCreatedAtRangeRule { min: 16, max: 32 }),
            Rc::new(ZoneQuotaLabelLengthRule { max_length: 86400 }),
            Rc::new(ZoneQuotaChannelLengthRule { max_length: 86400 }),
            Rc::new(ZoneQuotaCeilingRangeRule { min: 9.8, max: 1330.5 }),
        ],
    }
}
