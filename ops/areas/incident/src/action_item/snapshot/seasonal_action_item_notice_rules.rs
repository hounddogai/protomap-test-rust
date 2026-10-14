use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal action item notice must pass. Check returns its problems, or none.
pub trait SeasonalActionItemNoticeRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalActionItemNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemNoticeTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl SeasonalActionItemNoticeRule for SeasonalActionItemNoticeTagsLimitRule {
    fn check(&self, item: &SeasonalActionItemNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemNoticeKeyIdLengthRule {
    pub max_length: i64,
}

impl SeasonalActionItemNoticeRule for SeasonalActionItemNoticeKeyIdLengthRule {
    fn check(&self, item: &SeasonalActionItemNotice) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemNoticeRequiredKeyIdRule {
    pub message: String,
}

impl SeasonalActionItemNoticeRule for SeasonalActionItemNoticeRequiredKeyIdRule {
    fn check(&self, item: &SeasonalActionItemNotice) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemNoticeScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalActionItemNoticeRule for SeasonalActionItemNoticeScoreRangeRule {
    fn check(&self, item: &SeasonalActionItemNotice) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemNoticeTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalActionItemNoticeRule for SeasonalActionItemNoticeTtlSecondsRangeRule {
    fn check(&self, item: &SeasonalActionItemNotice) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal action item notice.
#[derive(Debug, Clone)]
pub struct SeasonalActionItemNoticeValidator {
    pub rules: Vec<Rc<dyn SeasonalActionItemNoticeRule>>,
}

impl SeasonalActionItemNoticeValidator {
    pub fn validate(&self, item: &SeasonalActionItemNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalActionItemNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalActionItemNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_action_item_notice_validator() -> SeasonalActionItemNoticeValidator {
    SeasonalActionItemNoticeValidator {
        rules: vec![
            Rc::new(SeasonalActionItemNoticeTagsLimitRule {
                max_count: 32,
                banned: Vec::new(),
            }) as Rc<dyn SeasonalActionItemNoticeRule>,
            Rc::new(SeasonalActionItemNoticeKeyIdLengthRule { max_length: 32 }),
            Rc::new(SeasonalActionItemNoticeRequiredKeyIdRule {
                message: "seasonal action item notice needs a value".to_string(),
            }),
            Rc::new(SeasonalActionItemNoticeScoreRangeRule { min: 6.2, max: 995.3 }),
            Rc::new(SeasonalActionItemNoticeTtlSecondsRangeRule { min: 16, max: 1000 }),
        ],
    }
}
