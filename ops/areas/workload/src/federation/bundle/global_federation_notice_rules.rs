use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global federation notice must pass. Check returns its problems, or none.
pub trait GlobalFederationNoticeRule: std::fmt::Debug {
    fn check(&self, item: &GlobalFederationNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationNoticeCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalFederationNoticeRule for GlobalFederationNoticeCeilingRangeRule {
    fn check(&self, item: &GlobalFederationNotice) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationNoticeSegmentLengthRule {
    pub max_length: i64,
}

impl GlobalFederationNoticeRule for GlobalFederationNoticeSegmentLengthRule {
    fn check(&self, item: &GlobalFederationNotice) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationNoticeRequiredNotesRule {
    pub message: String,
}

impl GlobalFederationNoticeRule for GlobalFederationNoticeRequiredNotesRule {
    fn check(&self, item: &GlobalFederationNotice) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global federation notice.
#[derive(Debug, Clone)]
pub struct GlobalFederationNoticeValidator {
    pub rules: Vec<Rc<dyn GlobalFederationNoticeRule>>,
}

impl GlobalFederationNoticeValidator {
    pub fn validate(&self, item: &GlobalFederationNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalFederationNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalFederationNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_federation_notice_validator() -> GlobalFederationNoticeValidator {
    GlobalFederationNoticeValidator {
        rules: vec![
            Rc::new(GlobalFederationNoticeCeilingRangeRule { min: 0.1, max: 4784.9 })
                as Rc<dyn GlobalFederationNoticeRule>,
            Rc::new(GlobalFederationNoticeSegmentLengthRule { max_length: 80 }),
            Rc::new(GlobalFederationNoticeRequiredNotesRule {
                message: "global federation notice needs a value".to_string(),
            }),
        ],
    }
}
