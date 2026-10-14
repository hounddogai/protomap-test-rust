use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem profile must pass. Check returns its problems, or none.
pub trait PostmortemProfileRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProfileCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemProfileRule for PostmortemProfileCodeLengthRule {
    fn check(&self, item: &PostmortemProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProfileCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemProfileRule for PostmortemProfileCapacityRangeRule {
    fn check(&self, item: &PostmortemProfile) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProfileAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PostmortemProfileRule for PostmortemProfileAllowedNameRule {
    fn check(&self, item: &PostmortemProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem profile.
#[derive(Debug, Clone)]
pub struct PostmortemProfileValidator {
    pub rules: Vec<Rc<dyn PostmortemProfileRule>>,
}

impl PostmortemProfileValidator {
    pub fn validate(&self, item: &PostmortemProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_profile_validator() -> PostmortemProfileValidator {
    PostmortemProfileValidator {
        rules: vec![
            Rc::new(PostmortemProfileCodeLengthRule { max_length: 16 }) as Rc<dyn PostmortemProfileRule>,
            Rc::new(PostmortemProfileCapacityRangeRule { min: 16, max: 32 }),
            Rc::new(PostmortemProfileAllowedNameRule {
                allowed: vec!["primary".to_string(), "fallback".to_string()],
            }),
        ],
    }
}
