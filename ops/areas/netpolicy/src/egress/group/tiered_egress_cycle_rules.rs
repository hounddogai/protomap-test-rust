use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered egress cycle must pass. Check returns its problems, or none.
pub trait TieredEgressCycleRule: std::fmt::Debug {
    fn check(&self, item: &TieredEgressCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressCycleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredEgressCycleRule for TieredEgressCycleRevisionRangeRule {
    fn check(&self, item: &TieredEgressCycle) -> Vec<String> {
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
pub struct TieredEgressCycleAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl TieredEgressCycleRule for TieredEgressCycleAllowedSegmentRule {
    fn check(&self, item: &TieredEgressCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is expired", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressCycleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredEgressCycleRule for TieredEgressCycleMinimumRangeRule {
    fn check(&self, item: &TieredEgressCycle) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressCycleSegmentLengthRule {
    pub max_length: i64,
}

impl TieredEgressCycleRule for TieredEgressCycleSegmentLengthRule {
    fn check(&self, item: &TieredEgressCycle) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressCycleRequiredNameRule {
    pub message: String,
}

impl TieredEgressCycleRule for TieredEgressCycleRequiredNameRule {
    fn check(&self, item: &TieredEgressCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressCycleCodePrefixRule {
    pub prefix: String,
}

impl TieredEgressCycleRule for TieredEgressCycleCodePrefixRule {
    fn check(&self, item: &TieredEgressCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered egress cycle.
#[derive(Debug, Clone)]
pub struct TieredEgressCycleValidator {
    pub rules: Vec<Rc<dyn TieredEgressCycleRule>>,
}

impl TieredEgressCycleValidator {
    pub fn validate(&self, item: &TieredEgressCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredEgressCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredEgressCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_egress_cycle_validator() -> TieredEgressCycleValidator {
    TieredEgressCycleValidator {
        rules: vec![
            Rc::new(TieredEgressCycleRevisionRangeRule { min: 120, max: 1000 }) as Rc<dyn TieredEgressCycleRule>,
            Rc::new(TieredEgressCycleAllowedSegmentRule { allowed: Vec::new() }),
            Rc::new(TieredEgressCycleMinimumRangeRule { min: 4.5, max: 4354.1 }),
            Rc::new(TieredEgressCycleSegmentLengthRule { max_length: 16 }),
            Rc::new(TieredEgressCycleRequiredNameRule {
                message: "tiered egress cycle needs this field".to_string(),
            }),
            Rc::new(TieredEgressCycleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
