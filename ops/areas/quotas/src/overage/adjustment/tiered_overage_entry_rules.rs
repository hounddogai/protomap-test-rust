use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered overage entry must pass. Check returns its problems, or none.
pub trait TieredOverageEntryRule: std::fmt::Debug {
    fn check(&self, item: &TieredOverageEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageEntrySegmentLengthRule {
    pub max_length: i64,
}

impl TieredOverageEntryRule for TieredOverageEntrySegmentLengthRule {
    fn check(&self, item: &TieredOverageEntry) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageEntryRequiredSegmentRule {
    pub message: String,
}

impl TieredOverageEntryRule for TieredOverageEntryRequiredSegmentRule {
    fn check(&self, item: &TieredOverageEntry) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageEntryRequiredChecksumRule {
    pub message: String,
}

impl TieredOverageEntryRule for TieredOverageEntryRequiredChecksumRule {
    fn check(&self, item: &TieredOverageEntry) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageEntryAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TieredOverageEntryRule for TieredOverageEntryAllowedTitleRule {
    fn check(&self, item: &TieredOverageEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered overage entry.
#[derive(Debug, Clone)]
pub struct TieredOverageEntryValidator {
    pub rules: Vec<Rc<dyn TieredOverageEntryRule>>,
}

impl TieredOverageEntryValidator {
    pub fn validate(&self, item: &TieredOverageEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredOverageEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredOverageEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_overage_entry_validator() -> TieredOverageEntryValidator {
    TieredOverageEntryValidator {
        rules: vec![
            Rc::new(TieredOverageEntrySegmentLengthRule { max_length: 80 }) as Rc<dyn TieredOverageEntryRule>,
            Rc::new(TieredOverageEntryRequiredSegmentRule {
                message: "tiered overage entry needs an entry".to_string(),
            }),
            Rc::new(TieredOverageEntryRequiredChecksumRule {
                message: "tiered overage entry needs an entry".to_string(),
            }),
            Rc::new(TieredOverageEntryAllowedTitleRule {
                allowed: vec!["rolling".to_string(), "global".to_string(), "tiered".to_string()],
            }),
        ],
    }
}
