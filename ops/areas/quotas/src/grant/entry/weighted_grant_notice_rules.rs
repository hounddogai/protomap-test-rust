use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted grant notice must pass. Check returns its problems, or none.
pub trait WeightedGrantNoticeRule: std::fmt::Debug {
    fn check(&self, item: &WeightedGrantNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantNoticeRequiredCodeRule {
    pub message: String,
}

impl WeightedGrantNoticeRule for WeightedGrantNoticeRequiredCodeRule {
    fn check(&self, item: &WeightedGrantNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantNoticeLabelLengthRule {
    pub max_length: i64,
}

impl WeightedGrantNoticeRule for WeightedGrantNoticeLabelLengthRule {
    fn check(&self, item: &WeightedGrantNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantNoticeMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedGrantNoticeRule for WeightedGrantNoticeMinimumRangeRule {
    fn check(&self, item: &WeightedGrantNotice) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantNoticeCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedGrantNoticeRule for WeightedGrantNoticeCreatedAtRangeRule {
    fn check(&self, item: &WeightedGrantNotice) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted grant notice.
#[derive(Debug, Clone)]
pub struct WeightedGrantNoticeValidator {
    pub rules: Vec<Rc<dyn WeightedGrantNoticeRule>>,
}

impl WeightedGrantNoticeValidator {
    pub fn validate(&self, item: &WeightedGrantNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedGrantNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedGrantNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_grant_notice_validator() -> WeightedGrantNoticeValidator {
    WeightedGrantNoticeValidator {
        rules: vec![
            Rc::new(WeightedGrantNoticeRequiredCodeRule {
                message: "weighted grant notice needs a value".to_string(),
            }) as Rc<dyn WeightedGrantNoticeRule>,
            Rc::new(WeightedGrantNoticeLabelLengthRule { max_length: 86400 }),
            Rc::new(WeightedGrantNoticeMinimumRangeRule { min: 5.6, max: 3626.0 }),
            Rc::new(WeightedGrantNoticeCreatedAtRangeRule { min: 16, max: 64 }),
        ],
    }
}
