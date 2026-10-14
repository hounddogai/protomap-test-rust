use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation notice must pass. Check returns its problems, or none.
pub trait IsolationNoticeRule: std::fmt::Debug {
    fn check(&self, item: &IsolationNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationNoticePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationNoticeRule for IsolationNoticePriorityRangeRule {
    fn check(&self, item: &IsolationNotice) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationNoticeNameLengthRule {
    pub max_length: i64,
}

impl IsolationNoticeRule for IsolationNoticeNameLengthRule {
    fn check(&self, item: &IsolationNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationNoticeCodeLengthRule {
    pub max_length: i64,
}

impl IsolationNoticeRule for IsolationNoticeCodeLengthRule {
    fn check(&self, item: &IsolationNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation notice.
#[derive(Debug, Clone)]
pub struct IsolationNoticeValidator {
    pub rules: Vec<Rc<dyn IsolationNoticeRule>>,
}

impl IsolationNoticeValidator {
    pub fn validate(&self, item: &IsolationNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_notice_validator() -> IsolationNoticeValidator {
    IsolationNoticeValidator {
        rules: vec![
            Rc::new(IsolationNoticePriorityRangeRule { min: 120, max: 3600 }) as Rc<dyn IsolationNoticeRule>,
            Rc::new(IsolationNoticeNameLengthRule { max_length: 32 }),
            Rc::new(IsolationNoticeCodeLengthRule { max_length: 16 }),
        ],
    }
}
