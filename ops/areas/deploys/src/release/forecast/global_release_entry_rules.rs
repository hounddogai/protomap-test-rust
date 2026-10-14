use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global release entry must pass. Check returns its problems, or none.
pub trait GlobalReleaseEntryRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReleaseEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEntryRequiredSummaryRule {
    pub message: String,
}

impl GlobalReleaseEntryRule for GlobalReleaseEntryRequiredSummaryRule {
    fn check(&self, item: &GlobalReleaseEntry) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEntrySegmentLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseEntryRule for GlobalReleaseEntrySegmentLengthRule {
    fn check(&self, item: &GlobalReleaseEntry) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEntryFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalReleaseEntryRule for GlobalReleaseEntryFloorValueRangeRule {
    fn check(&self, item: &GlobalReleaseEntry) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEntryRequiredLabelRule {
    pub message: String,
}

impl GlobalReleaseEntryRule for GlobalReleaseEntryRequiredLabelRule {
    fn check(&self, item: &GlobalReleaseEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEntryCodeLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseEntryRule for GlobalReleaseEntryCodeLengthRule {
    fn check(&self, item: &GlobalReleaseEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEntryRequiredCodeRule {
    pub message: String,
}

impl GlobalReleaseEntryRule for GlobalReleaseEntryRequiredCodeRule {
    fn check(&self, item: &GlobalReleaseEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global release entry.
#[derive(Debug, Clone)]
pub struct GlobalReleaseEntryValidator {
    pub rules: Vec<Rc<dyn GlobalReleaseEntryRule>>,
}

impl GlobalReleaseEntryValidator {
    pub fn validate(&self, item: &GlobalReleaseEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalReleaseEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReleaseEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_release_entry_validator() -> GlobalReleaseEntryValidator {
    GlobalReleaseEntryValidator {
        rules: vec![
            Rc::new(GlobalReleaseEntryRequiredSummaryRule {
                message: "global release entry needs this field".to_string(),
            }) as Rc<dyn GlobalReleaseEntryRule>,
            Rc::new(GlobalReleaseEntrySegmentLengthRule { max_length: 1000 }),
            Rc::new(GlobalReleaseEntryFloorValueRangeRule { min: 6.1, max: 721.1 }),
            Rc::new(GlobalReleaseEntryRequiredLabelRule {
                message: "global release entry needs a value".to_string(),
            }),
            Rc::new(GlobalReleaseEntryCodeLengthRule { max_length: 32 }),
            Rc::new(GlobalReleaseEntryRequiredCodeRule {
                message: "global release entry needs a value".to_string(),
            }),
        ],
    }
}
