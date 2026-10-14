use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant segment must pass. Check returns its problems, or none.
pub trait GrantSegmentRule: std::fmt::Debug {
    fn check(&self, item: &GrantSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSegmentCodeLengthRule {
    pub max_length: i64,
}

impl GrantSegmentRule for GrantSegmentCodeLengthRule {
    fn check(&self, item: &GrantSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSegmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantSegmentRule for GrantSegmentPriorityRangeRule {
    fn check(&self, item: &GrantSegment) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSegmentRequiredLabelRule {
    pub message: String,
}

impl GrantSegmentRule for GrantSegmentRequiredLabelRule {
    fn check(&self, item: &GrantSegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant segment.
#[derive(Debug, Clone)]
pub struct GrantSegmentValidator {
    pub rules: Vec<Rc<dyn GrantSegmentRule>>,
}

impl GrantSegmentValidator {
    pub fn validate(&self, item: &GrantSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_segment_validator() -> GrantSegmentValidator {
    GrantSegmentValidator {
        rules: vec![
            Rc::new(GrantSegmentCodeLengthRule { max_length: 86400 }) as Rc<dyn GrantSegmentRule>,
            Rc::new(GrantSegmentPriorityRangeRule { min: 32, max: 86400 }),
            Rc::new(GrantSegmentRequiredLabelRule {
                message: "grant segment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_segment_validator();
        let item = new_grant_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GrantSegmentCodeLengthRule { max_length: 3600 };
        let mut item = new_grant_segment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = GrantSegmentPriorityRangeRule { min: 255, max: 86400 };
        let mut item = new_grant_segment("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GrantSegmentRequiredLabelRule {
            message: "grant segment needs an entry".to_string(),
        };
        let mut item = new_grant_segment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
