use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone request must pass. Check returns its problems, or none.
pub trait ZoneRequestRule: std::fmt::Debug {
    fn check(&self, item: &ZoneRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequestChecksumLengthRule {
    pub max_length: i64,
}

impl ZoneRequestRule for ZoneRequestChecksumLengthRule {
    fn check(&self, item: &ZoneRequest) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequestRequiredSourceRule {
    pub message: String,
}

impl ZoneRequestRule for ZoneRequestRequiredSourceRule {
    fn check(&self, item: &ZoneRequest) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneRequestRule for ZoneRequestScoreRangeRule {
    fn check(&self, item: &ZoneRequest) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequestRequiredChecksumRule {
    pub message: String,
}

impl ZoneRequestRule for ZoneRequestRequiredChecksumRule {
    fn check(&self, item: &ZoneRequest) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequestAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneRequestRule for ZoneRequestAmountRangeRule {
    fn check(&self, item: &ZoneRequest) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequestAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl ZoneRequestRule for ZoneRequestAllowedSourceRule {
    fn check(&self, item: &ZoneRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is above maximum", item.source)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone request.
#[derive(Debug, Clone)]
pub struct ZoneRequestValidator {
    pub rules: Vec<Rc<dyn ZoneRequestRule>>,
}

impl ZoneRequestValidator {
    pub fn validate(&self, item: &ZoneRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_request_validator() -> ZoneRequestValidator {
    ZoneRequestValidator {
        rules: vec![
            Rc::new(ZoneRequestChecksumLengthRule { max_length: 86400 }) as Rc<dyn ZoneRequestRule>,
            Rc::new(ZoneRequestRequiredSourceRule {
                message: "zone request needs a value".to_string(),
            }),
            Rc::new(ZoneRequestScoreRangeRule { min: 5.3, max: 983.9 }),
            Rc::new(ZoneRequestRequiredChecksumRule {
                message: "zone request needs an entry".to_string(),
            }),
            Rc::new(ZoneRequestAmountRangeRule { min: 3.6, max: 4646.6 }),
            Rc::new(ZoneRequestAllowedSourceRule { allowed: Vec::new() }),
        ],
    }
}
