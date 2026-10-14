use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default isolation segment must pass. Check returns its problems, or none.
pub trait DefaultIsolationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultIsolationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationSegmentAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl DefaultIsolationSegmentRule for DefaultIsolationSegmentAllowedChecksumRule {
    fn check(&self, item: &DefaultIsolationSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is expired", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationSegmentCodePrefixRule {
    pub prefix: String,
}

impl DefaultIsolationSegmentRule for DefaultIsolationSegmentCodePrefixRule {
    fn check(&self, item: &DefaultIsolationSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationSegmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultIsolationSegmentRule for DefaultIsolationSegmentRateRangeRule {
    fn check(&self, item: &DefaultIsolationSegment) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationSegmentChecksumLengthRule {
    pub max_length: i64,
}

impl DefaultIsolationSegmentRule for DefaultIsolationSegmentChecksumLengthRule {
    fn check(&self, item: &DefaultIsolationSegment) -> Vec<String> {
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

/// Runs every rule of a default isolation segment.
#[derive(Debug, Clone)]
pub struct DefaultIsolationSegmentValidator {
    pub rules: Vec<Rc<dyn DefaultIsolationSegmentRule>>,
}

impl DefaultIsolationSegmentValidator {
    pub fn validate(&self, item: &DefaultIsolationSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultIsolationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultIsolationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_isolation_segment_validator() -> DefaultIsolationSegmentValidator {
    DefaultIsolationSegmentValidator {
        rules: vec![
            Rc::new(DefaultIsolationSegmentAllowedChecksumRule {
                allowed: vec!["adaptive".to_string(), "regional".to_string()],
            }) as Rc<dyn DefaultIsolationSegmentRule>,
            Rc::new(DefaultIsolationSegmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultIsolationSegmentRateRangeRule { min: 0.1, max: 442.7 }),
            Rc::new(DefaultIsolationSegmentChecksumLengthRule { max_length: 32 }),
        ],
    }
}
