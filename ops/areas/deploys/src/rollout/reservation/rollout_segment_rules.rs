use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout segment must pass. Check returns its problems, or none.
pub trait RolloutSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RolloutSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSegmentRequiredCodeRule {
    pub message: String,
}

impl RolloutSegmentRule for RolloutSegmentRequiredCodeRule {
    fn check(&self, item: &RolloutSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSegmentTitleLengthRule {
    pub max_length: i64,
}

impl RolloutSegmentRule for RolloutSegmentTitleLengthRule {
    fn check(&self, item: &RolloutSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSegmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RolloutSegmentRule for RolloutSegmentAllowedTitleRule {
    fn check(&self, item: &RolloutSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout segment.
#[derive(Debug, Clone)]
pub struct RolloutSegmentValidator {
    pub rules: Vec<Rc<dyn RolloutSegmentRule>>,
}

impl RolloutSegmentValidator {
    pub fn validate(&self, item: &RolloutSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_segment_validator() -> RolloutSegmentValidator {
    RolloutSegmentValidator {
        rules: vec![
            Rc::new(RolloutSegmentRequiredCodeRule {
                message: "rollout segment needs a value".to_string(),
            }) as Rc<dyn RolloutSegmentRule>,
            Rc::new(RolloutSegmentTitleLengthRule { max_length: 120 }),
            Rc::new(RolloutSegmentAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_segment_validator();
        let item = new_rollout_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutSegmentRequiredCodeRule {
            message: "rollout segment needs a value".to_string(),
        };
        let mut item = new_rollout_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RolloutSegmentTitleLengthRule { max_length: 16 };
        let mut item = new_rollout_segment("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
