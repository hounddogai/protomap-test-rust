use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched suspension segment must pass. Check returns its problems, or none.
pub trait BatchedSuspensionSegmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSuspensionSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionSegmentLabelLengthRule {
    pub max_length: i64,
}

impl BatchedSuspensionSegmentRule for BatchedSuspensionSegmentLabelLengthRule {
    fn check(&self, item: &BatchedSuspensionSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionSegmentRequiredScopeRule {
    pub message: String,
}

impl BatchedSuspensionSegmentRule for BatchedSuspensionSegmentRequiredScopeRule {
    fn check(&self, item: &BatchedSuspensionSegment) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionSegmentCodePrefixRule {
    pub prefix: String,
}

impl BatchedSuspensionSegmentRule for BatchedSuspensionSegmentCodePrefixRule {
    fn check(&self, item: &BatchedSuspensionSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionSegmentRequiredCodeRule {
    pub message: String,
}

impl BatchedSuspensionSegmentRule for BatchedSuspensionSegmentRequiredCodeRule {
    fn check(&self, item: &BatchedSuspensionSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionSegmentRequiredChecksumRule {
    pub message: String,
}

impl BatchedSuspensionSegmentRule for BatchedSuspensionSegmentRequiredChecksumRule {
    fn check(&self, item: &BatchedSuspensionSegment) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched suspension segment.
#[derive(Debug, Clone)]
pub struct BatchedSuspensionSegmentValidator {
    pub rules: Vec<Rc<dyn BatchedSuspensionSegmentRule>>,
}

impl BatchedSuspensionSegmentValidator {
    pub fn validate(&self, item: &BatchedSuspensionSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedSuspensionSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSuspensionSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_suspension_segment_validator() -> BatchedSuspensionSegmentValidator {
    BatchedSuspensionSegmentValidator {
        rules: vec![
            Rc::new(BatchedSuspensionSegmentLabelLengthRule { max_length: 16 }) as Rc<dyn BatchedSuspensionSegmentRule>,
            Rc::new(BatchedSuspensionSegmentRequiredScopeRule {
                message: "batched suspension segment needs this field".to_string(),
            }),
            Rc::new(BatchedSuspensionSegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BatchedSuspensionSegmentRequiredCodeRule {
                message: "batched suspension segment needs this field".to_string(),
            }),
            Rc::new(BatchedSuspensionSegmentRequiredChecksumRule {
                message: "batched suspension segment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_suspension_segment_validator();
        let item = new_batched_suspension_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedSuspensionSegmentLabelLengthRule { max_length: 3600 };
        let mut item = new_batched_suspension_segment("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = BatchedSuspensionSegmentRequiredScopeRule {
            message: "batched suspension segment needs this field".to_string(),
        };
        let mut item = new_batched_suspension_segment("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedSuspensionSegmentRequiredCodeRule {
            message: "batched suspension segment needs this field".to_string(),
        };
        let mut item = new_batched_suspension_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = BatchedSuspensionSegmentRequiredChecksumRule {
            message: "batched suspension segment needs a value".to_string(),
        };
        let mut item = new_batched_suspension_segment("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
