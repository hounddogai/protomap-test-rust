use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global quote entry must pass. Check returns its problems, or none.
pub trait GlobalQuoteEntryRule: std::fmt::Debug {
    fn check(&self, item: &GlobalQuoteEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteEntryRequiredLabelRule {
    pub message: String,
}

impl GlobalQuoteEntryRule for GlobalQuoteEntryRequiredLabelRule {
    fn check(&self, item: &GlobalQuoteEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteEntryCodePrefixRule {
    pub prefix: String,
}

impl GlobalQuoteEntryRule for GlobalQuoteEntryCodePrefixRule {
    fn check(&self, item: &GlobalQuoteEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteEntryBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalQuoteEntryRule for GlobalQuoteEntryBurstRangeRule {
    fn check(&self, item: &GlobalQuoteEntry) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteEntryStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalQuoteEntryRule for GlobalQuoteEntryStartsAtRangeRule {
    fn check(&self, item: &GlobalQuoteEntry) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global quote entry.
#[derive(Debug, Clone)]
pub struct GlobalQuoteEntryValidator {
    pub rules: Vec<Rc<dyn GlobalQuoteEntryRule>>,
}

impl GlobalQuoteEntryValidator {
    pub fn validate(&self, item: &GlobalQuoteEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalQuoteEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalQuoteEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_quote_entry_validator() -> GlobalQuoteEntryValidator {
    GlobalQuoteEntryValidator {
        rules: vec![
            Rc::new(GlobalQuoteEntryRequiredLabelRule {
                message: "global quote entry needs an entry".to_string(),
            }) as Rc<dyn GlobalQuoteEntryRule>,
            Rc::new(GlobalQuoteEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GlobalQuoteEntryBurstRangeRule { min: 16, max: 80 }),
            Rc::new(GlobalQuoteEntryStartsAtRangeRule { min: 80, max: 120 }),
        ],
    }
}
