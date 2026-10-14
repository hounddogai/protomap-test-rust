use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem notice must pass. Check returns its problems, or none.
pub trait PostmortemNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemNoticeRequiredCodeRule {
    pub message: String,
}

impl PostmortemNoticeRule for PostmortemNoticeRequiredCodeRule {
    fn check(&self, item: &PostmortemNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemNoticeRequiredTitleRule {
    pub message: String,
}

impl PostmortemNoticeRule for PostmortemNoticeRequiredTitleRule {
    fn check(&self, item: &PostmortemNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemNoticeRequiredOwnerIdRule {
    pub message: String,
}

impl PostmortemNoticeRule for PostmortemNoticeRequiredOwnerIdRule {
    fn check(&self, item: &PostmortemNotice) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemNoticeTitleLengthRule {
    pub max_length: i64,
}

impl PostmortemNoticeRule for PostmortemNoticeTitleLengthRule {
    fn check(&self, item: &PostmortemNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem notice.
#[derive(Debug, Clone)]
pub struct PostmortemNoticeValidator {
    pub rules: Vec<Rc<dyn PostmortemNoticeRule>>,
}

impl PostmortemNoticeValidator {
    pub fn validate(&self, item: &PostmortemNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_notice_validator() -> PostmortemNoticeValidator {
    PostmortemNoticeValidator {
        rules: vec![
            Rc::new(PostmortemNoticeRequiredCodeRule {
                message: "postmortem notice needs a value".to_string(),
            }) as Rc<dyn PostmortemNoticeRule>,
            Rc::new(PostmortemNoticeRequiredTitleRule {
                message: "postmortem notice needs an entry".to_string(),
            }),
            Rc::new(PostmortemNoticeRequiredOwnerIdRule {
                message: "postmortem notice needs this field".to_string(),
            }),
            Rc::new(PostmortemNoticeTitleLengthRule { max_length: 120 }),
        ],
    }
}
