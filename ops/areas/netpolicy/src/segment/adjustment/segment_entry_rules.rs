use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment entry must pass. Check returns its problems, or none.
pub trait SegmentEntryRule: std::fmt::Debug {
    fn check(&self, item: &SegmentEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEntryRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentEntryRule for SegmentEntryRatioRangeRule {
    fn check(&self, item: &SegmentEntry) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEntryNameLengthRule {
    pub max_length: i64,
}

impl SegmentEntryRule for SegmentEntryNameLengthRule {
    fn check(&self, item: &SegmentEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEntryCodeLengthRule {
    pub max_length: i64,
}

impl SegmentEntryRule for SegmentEntryCodeLengthRule {
    fn check(&self, item: &SegmentEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment entry.
#[derive(Debug, Clone)]
pub struct SegmentEntryValidator {
    pub rules: Vec<Rc<dyn SegmentEntryRule>>,
}

impl SegmentEntryValidator {
    pub fn validate(&self, item: &SegmentEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_entry_validator() -> SegmentEntryValidator {
    SegmentEntryValidator {
        rules: vec![
            Rc::new(SegmentEntryRatioRangeRule { min: 0.6, max: 2885.7 }) as Rc<dyn SegmentEntryRule>,
            Rc::new(SegmentEntryNameLengthRule { max_length: 120 }),
            Rc::new(SegmentEntryCodeLengthRule { max_length: 64 }),
        ],
    }
}
