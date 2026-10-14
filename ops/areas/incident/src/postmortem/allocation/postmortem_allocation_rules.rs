use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem allocation must pass. Check returns its problems, or none.
pub trait PostmortemAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAllocationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemAllocationRule for PostmortemAllocationGraceDaysRangeRule {
    fn check(&self, item: &PostmortemAllocation) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAllocationRequiredTitleRule {
    pub message: String,
}

impl PostmortemAllocationRule for PostmortemAllocationRequiredTitleRule {
    fn check(&self, item: &PostmortemAllocation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAllocationTitleLengthRule {
    pub max_length: i64,
}

impl PostmortemAllocationRule for PostmortemAllocationTitleLengthRule {
    fn check(&self, item: &PostmortemAllocation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAllocationMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemAllocationRule for PostmortemAllocationMaximumRangeRule {
    fn check(&self, item: &PostmortemAllocation) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAllocationCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemAllocationRule for PostmortemAllocationCodeLengthRule {
    fn check(&self, item: &PostmortemAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem allocation.
#[derive(Debug, Clone)]
pub struct PostmortemAllocationValidator {
    pub rules: Vec<Rc<dyn PostmortemAllocationRule>>,
}

impl PostmortemAllocationValidator {
    pub fn validate(&self, item: &PostmortemAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_allocation_validator() -> PostmortemAllocationValidator {
    PostmortemAllocationValidator {
        rules: vec![
            Rc::new(PostmortemAllocationGraceDaysRangeRule { min: 80, max: 120 }) as Rc<dyn PostmortemAllocationRule>,
            Rc::new(PostmortemAllocationRequiredTitleRule {
                message: "postmortem allocation needs this field".to_string(),
            }),
            Rc::new(PostmortemAllocationTitleLengthRule { max_length: 32 }),
            Rc::new(PostmortemAllocationMaximumRangeRule { min: 5.4, max: 4246.1 }),
            Rc::new(PostmortemAllocationCodeLengthRule { max_length: 32 }),
        ],
    }
}
