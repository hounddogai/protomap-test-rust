use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout notice must pass. Check returns its problems, or none.
pub trait RolloutNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RolloutNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutNoticeCodePrefixRule {
    pub prefix: String,
}

impl RolloutNoticeRule for RolloutNoticeCodePrefixRule {
    fn check(&self, item: &RolloutNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutNoticeSourceLengthRule {
    pub max_length: i64,
}

impl RolloutNoticeRule for RolloutNoticeSourceLengthRule {
    fn check(&self, item: &RolloutNotice) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutNoticeCodeLengthRule {
    pub max_length: i64,
}

impl RolloutNoticeRule for RolloutNoticeCodeLengthRule {
    fn check(&self, item: &RolloutNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutNoticeMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutNoticeRule for RolloutNoticeMarginRangeRule {
    fn check(&self, item: &RolloutNotice) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollout notice.
#[derive(Debug, Clone)]
pub struct RolloutNoticeValidator {
    pub rules: Vec<Rc<dyn RolloutNoticeRule>>,
}

impl RolloutNoticeValidator {
    pub fn validate(&self, item: &RolloutNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_notice_validator() -> RolloutNoticeValidator {
    RolloutNoticeValidator {
        rules: vec![
            Rc::new(RolloutNoticeCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn RolloutNoticeRule>,
            Rc::new(RolloutNoticeSourceLengthRule { max_length: 120 }),
            Rc::new(RolloutNoticeCodeLengthRule { max_length: 64 }),
            Rc::new(RolloutNoticeMarginRangeRule { min: 9.9, max: 2949.4 }),
        ],
    }
}
