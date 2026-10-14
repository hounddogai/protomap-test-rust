use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page record must pass. Check returns its problems, or none.
pub trait PageRecordRule: std::fmt::Debug {
    fn check(&self, item: &PageRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRecordRequiredDescriptionRule {
    pub message: String,
}

impl PageRecordRule for PageRecordRequiredDescriptionRule {
    fn check(&self, item: &PageRecord) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRecordUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageRecordRule for PageRecordUpdatedAtRangeRule {
    fn check(&self, item: &PageRecord) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRecordPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageRecordRule for PageRecordPortRangeRule {
    fn check(&self, item: &PageRecord) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page record.
#[derive(Debug, Clone)]
pub struct PageRecordValidator {
    pub rules: Vec<Rc<dyn PageRecordRule>>,
}

impl PageRecordValidator {
    pub fn validate(&self, item: &PageRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_record_validator() -> PageRecordValidator {
    PageRecordValidator {
        rules: vec![
            Rc::new(PageRecordRequiredDescriptionRule {
                message: "page record needs this field".to_string(),
            }) as Rc<dyn PageRecordRule>,
            Rc::new(PageRecordUpdatedAtRangeRule { min: 120, max: 3600 }),
            Rc::new(PageRecordPortRangeRule { min: 16, max: 86400 }),
        ],
    }
}
