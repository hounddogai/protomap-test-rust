use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule notice must pass. Check returns its problems, or none.
pub trait RuleNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RuleNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleNoticePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleNoticeRule for RuleNoticePortRangeRule {
    fn check(&self, item: &RuleNotice) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleNoticeNameLengthRule {
    pub max_length: i64,
}

impl RuleNoticeRule for RuleNoticeNameLengthRule {
    fn check(&self, item: &RuleNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleNoticeCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleNoticeRule for RuleNoticeCapacityRangeRule {
    fn check(&self, item: &RuleNotice) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleNoticeRequiredFingerprintRule {
    pub message: String,
}

impl RuleNoticeRule for RuleNoticeRequiredFingerprintRule {
    fn check(&self, item: &RuleNotice) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule notice.
#[derive(Debug, Clone)]
pub struct RuleNoticeValidator {
    pub rules: Vec<Rc<dyn RuleNoticeRule>>,
}

impl RuleNoticeValidator {
    pub fn validate(&self, item: &RuleNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_notice_validator() -> RuleNoticeValidator {
    RuleNoticeValidator {
        rules: vec![
            Rc::new(RuleNoticePortRangeRule { min: 120, max: 3600 }) as Rc<dyn RuleNoticeRule>,
            Rc::new(RuleNoticeNameLengthRule { max_length: 1000 }),
            Rc::new(RuleNoticeCapacityRangeRule { min: 120, max: 3600 }),
            Rc::new(RuleNoticeRequiredFingerprintRule {
                message: "rule notice needs this field".to_string(),
            }),
        ],
    }
}
