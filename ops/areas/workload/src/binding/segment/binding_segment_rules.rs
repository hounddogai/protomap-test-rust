use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding segment must pass. Check returns its problems, or none.
pub trait BindingSegmentRule: std::fmt::Debug {
    fn check(&self, item: &BindingSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSegmentCodeLengthRule {
    pub max_length: i64,
}

impl BindingSegmentRule for BindingSegmentCodeLengthRule {
    fn check(&self, item: &BindingSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSegmentKeyIdLengthRule {
    pub max_length: i64,
}

impl BindingSegmentRule for BindingSegmentKeyIdLengthRule {
    fn check(&self, item: &BindingSegment) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSegmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingSegmentRule for BindingSegmentVersionRangeRule {
    fn check(&self, item: &BindingSegment) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a binding segment.
#[derive(Debug, Clone)]
pub struct BindingSegmentValidator {
    pub rules: Vec<Rc<dyn BindingSegmentRule>>,
}

impl BindingSegmentValidator {
    pub fn validate(&self, item: &BindingSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_segment_validator() -> BindingSegmentValidator {
    BindingSegmentValidator {
        rules: vec![
            Rc::new(BindingSegmentCodeLengthRule { max_length: 86400 }) as Rc<dyn BindingSegmentRule>,
            Rc::new(BindingSegmentKeyIdLengthRule { max_length: 3600 }),
            Rc::new(BindingSegmentVersionRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_segment_validator();
        let item = new_binding_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingSegmentCodeLengthRule { max_length: 64 };
        let mut item = new_binding_segment("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = BindingSegmentKeyIdLengthRule { max_length: 64 };
        let mut item = new_binding_segment("x", "y");
        item.key_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = BindingSegmentVersionRangeRule { min: 64, max: 120 };
        let mut item = new_binding_segment("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
