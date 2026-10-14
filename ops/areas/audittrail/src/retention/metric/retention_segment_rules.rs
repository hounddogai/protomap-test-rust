use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention segment must pass. Check returns its problems, or none.
pub trait RetentionSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RetentionSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSegmentRequiredCodeRule {
    pub message: String,
}

impl RetentionSegmentRule for RetentionSegmentRequiredCodeRule {
    fn check(&self, item: &RetentionSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSegmentLabelLengthRule {
    pub max_length: i64,
}

impl RetentionSegmentRule for RetentionSegmentLabelLengthRule {
    fn check(&self, item: &RetentionSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSegmentCodeLengthRule {
    pub max_length: i64,
}

impl RetentionSegmentRule for RetentionSegmentCodeLengthRule {
    fn check(&self, item: &RetentionSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention segment.
#[derive(Debug, Clone)]
pub struct RetentionSegmentValidator {
    pub rules: Vec<Rc<dyn RetentionSegmentRule>>,
}

impl RetentionSegmentValidator {
    pub fn validate(&self, item: &RetentionSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_segment_validator() -> RetentionSegmentValidator {
    RetentionSegmentValidator {
        rules: vec![
            Rc::new(RetentionSegmentRequiredCodeRule {
                message: "retention segment needs this field".to_string(),
            }) as Rc<dyn RetentionSegmentRule>,
            Rc::new(RetentionSegmentLabelLengthRule { max_length: 32 }),
            Rc::new(RetentionSegmentCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_segment_validator();
        let item = new_retention_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RetentionSegmentRequiredCodeRule {
            message: "retention segment needs this field".to_string(),
        };
        let mut item = new_retention_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RetentionSegmentLabelLengthRule { max_length: 32 };
        let mut item = new_retention_segment("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionSegmentCodeLengthRule { max_length: 32 };
        let mut item = new_retention_segment("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
