use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient rule notice must pass. Check returns its problems, or none.
pub trait LenientRuleNoticeRule: std::fmt::Debug {
    fn check(&self, item: &LenientRuleNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleNoticeKeyIdLengthRule {
    pub max_length: i64,
}

impl LenientRuleNoticeRule for LenientRuleNoticeKeyIdLengthRule {
    fn check(&self, item: &LenientRuleNotice) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleNoticeWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRuleNoticeRule for LenientRuleNoticeWeightRangeRule {
    fn check(&self, item: &LenientRuleNotice) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleNoticeTitleLengthRule {
    pub max_length: i64,
}

impl LenientRuleNoticeRule for LenientRuleNoticeTitleLengthRule {
    fn check(&self, item: &LenientRuleNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleNoticeRequiredKeyIdRule {
    pub message: String,
}

impl LenientRuleNoticeRule for LenientRuleNoticeRequiredKeyIdRule {
    fn check(&self, item: &LenientRuleNotice) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient rule notice.
#[derive(Debug, Clone)]
pub struct LenientRuleNoticeValidator {
    pub rules: Vec<Rc<dyn LenientRuleNoticeRule>>,
}

impl LenientRuleNoticeValidator {
    pub fn validate(&self, item: &LenientRuleNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRuleNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRuleNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_rule_notice_validator() -> LenientRuleNoticeValidator {
    LenientRuleNoticeValidator {
        rules: vec![
            Rc::new(LenientRuleNoticeKeyIdLengthRule { max_length: 86400 }) as Rc<dyn LenientRuleNoticeRule>,
            Rc::new(LenientRuleNoticeWeightRangeRule { min: 8.8, max: 2587.4 }),
            Rc::new(LenientRuleNoticeTitleLengthRule { max_length: 255 }),
            Rc::new(LenientRuleNoticeRequiredKeyIdRule {
                message: "lenient rule notice needs an entry".to_string(),
            }),
        ],
    }
}
