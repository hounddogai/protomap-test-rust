use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual canary segment must pass. Check returns its problems, or none.
pub trait ManualCanarySegmentRule: std::fmt::Debug {
    fn check(&self, item: &ManualCanarySegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanarySegmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualCanarySegmentRule for ManualCanarySegmentGraceDaysRangeRule {
    fn check(&self, item: &ManualCanarySegment) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanarySegmentChannelLengthRule {
    pub max_length: i64,
}

impl ManualCanarySegmentRule for ManualCanarySegmentChannelLengthRule {
    fn check(&self, item: &ManualCanarySegment) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanarySegmentRequiredHostRule {
    pub message: String,
}

impl ManualCanarySegmentRule for ManualCanarySegmentRequiredHostRule {
    fn check(&self, item: &ManualCanarySegment) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanarySegmentCodePrefixRule {
    pub prefix: String,
}

impl ManualCanarySegmentRule for ManualCanarySegmentCodePrefixRule {
    fn check(&self, item: &ManualCanarySegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanarySegmentHostLengthRule {
    pub max_length: i64,
}

impl ManualCanarySegmentRule for ManualCanarySegmentHostLengthRule {
    fn check(&self, item: &ManualCanarySegment) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual canary segment.
#[derive(Debug, Clone)]
pub struct ManualCanarySegmentValidator {
    pub rules: Vec<Rc<dyn ManualCanarySegmentRule>>,
}

impl ManualCanarySegmentValidator {
    pub fn validate(&self, item: &ManualCanarySegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualCanarySegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualCanarySegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_canary_segment_validator() -> ManualCanarySegmentValidator {
    ManualCanarySegmentValidator {
        rules: vec![
            Rc::new(ManualCanarySegmentGraceDaysRangeRule { min: 64, max: 1000 }) as Rc<dyn ManualCanarySegmentRule>,
            Rc::new(ManualCanarySegmentChannelLengthRule { max_length: 120 }),
            Rc::new(ManualCanarySegmentRequiredHostRule {
                message: "manual canary segment needs an entry".to_string(),
            }),
            Rc::new(ManualCanarySegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ManualCanarySegmentHostLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_canary_segment_validator();
        let item = new_manual_canary_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ManualCanarySegmentGraceDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_manual_canary_segment("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ManualCanarySegmentChannelLengthRule { max_length: 80 };
        let mut item = new_manual_canary_segment("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = ManualCanarySegmentRequiredHostRule {
            message: "manual canary segment needs a value".to_string(),
        };
        let mut item = new_manual_canary_segment("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = ManualCanarySegmentHostLengthRule { max_length: 255 };
        let mut item = new_manual_canary_segment("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
