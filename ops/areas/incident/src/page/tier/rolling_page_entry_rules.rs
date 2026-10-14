use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling page entry must pass. Check returns its problems, or none.
pub trait RollingPageEntryRule: std::fmt::Debug {
    fn check(&self, item: &RollingPageEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageEntryCodePrefixRule {
    pub prefix: String,
}

impl RollingPageEntryRule for RollingPageEntryCodePrefixRule {
    fn check(&self, item: &RollingPageEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageEntryAudienceLengthRule {
    pub max_length: i64,
}

impl RollingPageEntryRule for RollingPageEntryAudienceLengthRule {
    fn check(&self, item: &RollingPageEntry) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageEntryTitleLengthRule {
    pub max_length: i64,
}

impl RollingPageEntryRule for RollingPageEntryTitleLengthRule {
    fn check(&self, item: &RollingPageEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageEntryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingPageEntryRule for RollingPageEntryQuantityRangeRule {
    fn check(&self, item: &RollingPageEntry) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling page entry.
#[derive(Debug, Clone)]
pub struct RollingPageEntryValidator {
    pub rules: Vec<Rc<dyn RollingPageEntryRule>>,
}

impl RollingPageEntryValidator {
    pub fn validate(&self, item: &RollingPageEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingPageEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPageEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_page_entry_validator() -> RollingPageEntryValidator {
    RollingPageEntryValidator {
        rules: vec![
            Rc::new(RollingPageEntryCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RollingPageEntryRule>,
            Rc::new(RollingPageEntryAudienceLengthRule { max_length: 120 }),
            Rc::new(RollingPageEntryTitleLengthRule { max_length: 120 }),
            Rc::new(RollingPageEntryQuantityRangeRule { min: 80, max: 86400 }),
        ],
    }
}
