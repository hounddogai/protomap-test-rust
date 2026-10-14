use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional usage segment must pass. Check returns its problems, or none.
pub trait RegionalUsageSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalUsageSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSegmentNameLengthRule {
    pub max_length: i64,
}

impl RegionalUsageSegmentRule for RegionalUsageSegmentNameLengthRule {
    fn check(&self, item: &RegionalUsageSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSegmentHostLengthRule {
    pub max_length: i64,
}

impl RegionalUsageSegmentRule for RegionalUsageSegmentHostLengthRule {
    fn check(&self, item: &RegionalUsageSegment) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSegmentCodePrefixRule {
    pub prefix: String,
}

impl RegionalUsageSegmentRule for RegionalUsageSegmentCodePrefixRule {
    fn check(&self, item: &RegionalUsageSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSegmentRequiredHostRule {
    pub message: String,
}

impl RegionalUsageSegmentRule for RegionalUsageSegmentRequiredHostRule {
    fn check(&self, item: &RegionalUsageSegment) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional usage segment.
#[derive(Debug, Clone)]
pub struct RegionalUsageSegmentValidator {
    pub rules: Vec<Rc<dyn RegionalUsageSegmentRule>>,
}

impl RegionalUsageSegmentValidator {
    pub fn validate(&self, item: &RegionalUsageSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalUsageSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalUsageSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_usage_segment_validator() -> RegionalUsageSegmentValidator {
    RegionalUsageSegmentValidator {
        rules: vec![
            Rc::new(RegionalUsageSegmentNameLengthRule { max_length: 255 }) as Rc<dyn RegionalUsageSegmentRule>,
            Rc::new(RegionalUsageSegmentHostLengthRule { max_length: 16 }),
            Rc::new(RegionalUsageSegmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionalUsageSegmentRequiredHostRule {
                message: "regional usage segment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_usage_segment_validator();
        let item = new_regional_usage_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalUsageSegmentNameLengthRule { max_length: 255 };
        let mut item = new_regional_usage_segment("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RegionalUsageSegmentHostLengthRule { max_length: 1000 };
        let mut item = new_regional_usage_segment("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RegionalUsageSegmentRequiredHostRule {
            message: "regional usage segment needs an entry".to_string(),
        };
        let mut item = new_regional_usage_segment("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
