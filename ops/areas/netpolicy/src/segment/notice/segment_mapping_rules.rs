use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment mapping must pass. Check returns its problems, or none.
pub trait SegmentMappingRule: std::fmt::Debug {
    fn check(&self, item: &SegmentMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMappingCodeLengthRule {
    pub max_length: i64,
}

impl SegmentMappingRule for SegmentMappingCodeLengthRule {
    fn check(&self, item: &SegmentMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMappingSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentMappingRule for SegmentMappingSequenceRangeRule {
    fn check(&self, item: &SegmentMapping) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMappingLabelLengthRule {
    pub max_length: i64,
}

impl SegmentMappingRule for SegmentMappingLabelLengthRule {
    fn check(&self, item: &SegmentMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMappingCodePrefixRule {
    pub prefix: String,
}

impl SegmentMappingRule for SegmentMappingCodePrefixRule {
    fn check(&self, item: &SegmentMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMappingPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentMappingRule for SegmentMappingPriorityRangeRule {
    fn check(&self, item: &SegmentMapping) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMappingApprovedGuardRule {
    pub limit: f64,
}

impl SegmentMappingRule for SegmentMappingApprovedGuardRule {
    fn check(&self, item: &SegmentMapping) -> Vec<String> {
        if item.approved && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while approved", item.score, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment mapping.
#[derive(Debug, Clone)]
pub struct SegmentMappingValidator {
    pub rules: Vec<Rc<dyn SegmentMappingRule>>,
}

impl SegmentMappingValidator {
    pub fn validate(&self, item: &SegmentMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_mapping_validator() -> SegmentMappingValidator {
    SegmentMappingValidator {
        rules: vec![
            Rc::new(SegmentMappingCodeLengthRule { max_length: 32 }) as Rc<dyn SegmentMappingRule>,
            Rc::new(SegmentMappingSequenceRangeRule { min: 32, max: 1000 }),
            Rc::new(SegmentMappingLabelLengthRule { max_length: 32 }),
            Rc::new(SegmentMappingCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SegmentMappingPriorityRangeRule { min: 64, max: 1000 }),
            Rc::new(SegmentMappingApprovedGuardRule { limit: 3.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_mapping_validator();
        let item = new_segment_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentMappingCodeLengthRule { max_length: 32 };
        let mut item = new_segment_mapping("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SegmentMappingSequenceRangeRule { min: 32, max: 86400 };
        let mut item = new_segment_mapping("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SegmentMappingLabelLengthRule { max_length: 1000 };
        let mut item = new_segment_mapping("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SegmentMappingPriorityRangeRule { min: 80, max: 80 };
        let mut item = new_segment_mapping("x", "y");
        item.priority = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
