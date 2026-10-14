use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual rule profile must pass. Check returns its problems, or none.
pub trait ManualRuleProfileRule: std::fmt::Debug {
    fn check(&self, item: &ManualRuleProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleProfileCodeLengthRule {
    pub max_length: i64,
}

impl ManualRuleProfileRule for ManualRuleProfileCodeLengthRule {
    fn check(&self, item: &ManualRuleProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleProfileSegmentLengthRule {
    pub max_length: i64,
}

impl ManualRuleProfileRule for ManualRuleProfileSegmentLengthRule {
    fn check(&self, item: &ManualRuleProfile) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleProfileAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ManualRuleProfileRule for ManualRuleProfileAllowedTitleRule {
    fn check(&self, item: &ManualRuleProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleProfileTitleLengthRule {
    pub max_length: i64,
}

impl ManualRuleProfileRule for ManualRuleProfileTitleLengthRule {
    fn check(&self, item: &ManualRuleProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleProfileChannelLengthRule {
    pub max_length: i64,
}

impl ManualRuleProfileRule for ManualRuleProfileChannelLengthRule {
    fn check(&self, item: &ManualRuleProfile) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual rule profile.
#[derive(Debug, Clone)]
pub struct ManualRuleProfileValidator {
    pub rules: Vec<Rc<dyn ManualRuleProfileRule>>,
}

impl ManualRuleProfileValidator {
    pub fn validate(&self, item: &ManualRuleProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRuleProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRuleProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_rule_profile_validator() -> ManualRuleProfileValidator {
    ManualRuleProfileValidator {
        rules: vec![
            Rc::new(ManualRuleProfileCodeLengthRule { max_length: 1000 }) as Rc<dyn ManualRuleProfileRule>,
            Rc::new(ManualRuleProfileSegmentLengthRule { max_length: 255 }),
            Rc::new(ManualRuleProfileAllowedTitleRule {
                allowed: vec!["capped".to_string(), "regional".to_string(), "seasonal".to_string()],
            }),
            Rc::new(ManualRuleProfileTitleLengthRule { max_length: 64 }),
            Rc::new(ManualRuleProfileChannelLengthRule { max_length: 86400 }),
        ],
    }
}
