use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive usage segment must pass. Check returns its problems, or none.
pub trait AdaptiveUsageSegmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveUsageSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageSegmentRequiredScopeRule {
    pub message: String,
}

impl AdaptiveUsageSegmentRule for AdaptiveUsageSegmentRequiredScopeRule {
    fn check(&self, item: &AdaptiveUsageSegment) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageSegmentNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveUsageSegmentRule for AdaptiveUsageSegmentNameLengthRule {
    fn check(&self, item: &AdaptiveUsageSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageSegmentCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveUsageSegmentRule for AdaptiveUsageSegmentCodeLengthRule {
    fn check(&self, item: &AdaptiveUsageSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageSegmentEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveUsageSegmentRule for AdaptiveUsageSegmentEndsAtRangeRule {
    fn check(&self, item: &AdaptiveUsageSegment) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageSegmentRequiredNameRule {
    pub message: String,
}

impl AdaptiveUsageSegmentRule for AdaptiveUsageSegmentRequiredNameRule {
    fn check(&self, item: &AdaptiveUsageSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive usage segment.
#[derive(Debug, Clone)]
pub struct AdaptiveUsageSegmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveUsageSegmentRule>>,
}

impl AdaptiveUsageSegmentValidator {
    pub fn validate(&self, item: &AdaptiveUsageSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveUsageSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveUsageSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_usage_segment_validator() -> AdaptiveUsageSegmentValidator {
    AdaptiveUsageSegmentValidator {
        rules: vec![
            Rc::new(AdaptiveUsageSegmentRequiredScopeRule {
                message: "adaptive usage segment needs a value".to_string(),
            }) as Rc<dyn AdaptiveUsageSegmentRule>,
            Rc::new(AdaptiveUsageSegmentNameLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveUsageSegmentCodeLengthRule { max_length: 32 }),
            Rc::new(AdaptiveUsageSegmentEndsAtRangeRule { min: 3600, max: 3600 }),
            Rc::new(AdaptiveUsageSegmentRequiredNameRule {
                message: "adaptive usage segment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_usage_segment_validator();
        let item = new_adaptive_usage_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = AdaptiveUsageSegmentRequiredScopeRule {
            message: "adaptive usage segment needs an entry".to_string(),
        };
        let mut item = new_adaptive_usage_segment("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AdaptiveUsageSegmentNameLengthRule { max_length: 3600 };
        let mut item = new_adaptive_usage_segment("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveUsageSegmentCodeLengthRule { max_length: 32 };
        let mut item = new_adaptive_usage_segment("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = AdaptiveUsageSegmentEndsAtRangeRule { min: 64, max: 255 };
        let mut item = new_adaptive_usage_segment("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptiveUsageSegmentRequiredNameRule {
            message: "adaptive usage segment needs an entry".to_string(),
        };
        let mut item = new_adaptive_usage_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
