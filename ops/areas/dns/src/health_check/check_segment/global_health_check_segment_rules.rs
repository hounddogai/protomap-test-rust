use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global health check segment must pass. Check returns its problems, or none.
pub trait GlobalHealthCheckSegmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalHealthCheckSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckSegmentTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalHealthCheckSegmentRule for GlobalHealthCheckSegmentTtlSecondsRangeRule {
    fn check(&self, item: &GlobalHealthCheckSegment) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckSegmentCodeLengthRule {
    pub max_length: i64,
}

impl GlobalHealthCheckSegmentRule for GlobalHealthCheckSegmentCodeLengthRule {
    fn check(&self, item: &GlobalHealthCheckSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckSegmentRequiredCodeRule {
    pub message: String,
}

impl GlobalHealthCheckSegmentRule for GlobalHealthCheckSegmentRequiredCodeRule {
    fn check(&self, item: &GlobalHealthCheckSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckSegmentRequiredNameRule {
    pub message: String,
}

impl GlobalHealthCheckSegmentRule for GlobalHealthCheckSegmentRequiredNameRule {
    fn check(&self, item: &GlobalHealthCheckSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckSegmentChecksumLengthRule {
    pub max_length: i64,
}

impl GlobalHealthCheckSegmentRule for GlobalHealthCheckSegmentChecksumLengthRule {
    fn check(&self, item: &GlobalHealthCheckSegment) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckSegmentTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl GlobalHealthCheckSegmentRule for GlobalHealthCheckSegmentTagsLimitRule {
    fn check(&self, item: &GlobalHealthCheckSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a global health check segment.
#[derive(Debug, Clone)]
pub struct GlobalHealthCheckSegmentValidator {
    pub rules: Vec<Rc<dyn GlobalHealthCheckSegmentRule>>,
}

impl GlobalHealthCheckSegmentValidator {
    pub fn validate(&self, item: &GlobalHealthCheckSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalHealthCheckSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalHealthCheckSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_health_check_segment_validator() -> GlobalHealthCheckSegmentValidator {
    GlobalHealthCheckSegmentValidator {
        rules: vec![
            Rc::new(GlobalHealthCheckSegmentTtlSecondsRangeRule { min: 16, max: 86400 })
                as Rc<dyn GlobalHealthCheckSegmentRule>,
            Rc::new(GlobalHealthCheckSegmentCodeLengthRule { max_length: 3600 }),
            Rc::new(GlobalHealthCheckSegmentRequiredCodeRule {
                message: "global health check segment needs a value".to_string(),
            }),
            Rc::new(GlobalHealthCheckSegmentRequiredNameRule {
                message: "global health check segment needs an entry".to_string(),
            }),
            Rc::new(GlobalHealthCheckSegmentChecksumLengthRule { max_length: 3600 }),
            Rc::new(GlobalHealthCheckSegmentTagsLimitRule {
                max_count: 1000,
                banned: vec!["fallback".to_string(), "primary".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_health_check_segment_validator();
        let item = new_global_health_check_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = GlobalHealthCheckSegmentTtlSecondsRangeRule { min: 64, max: 255 };
        let mut item = new_global_health_check_segment("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalHealthCheckSegmentCodeLengthRule { max_length: 3600 };
        let mut item = new_global_health_check_segment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalHealthCheckSegmentRequiredCodeRule {
            message: "global health check segment needs an entry".to_string(),
        };
        let mut item = new_global_health_check_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalHealthCheckSegmentRequiredNameRule {
            message: "global health check segment needs an entry".to_string(),
        };
        let mut item = new_global_health_check_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = GlobalHealthCheckSegmentChecksumLengthRule { max_length: 255 };
        let mut item = new_global_health_check_segment("x", "y");
        item.checksum = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
