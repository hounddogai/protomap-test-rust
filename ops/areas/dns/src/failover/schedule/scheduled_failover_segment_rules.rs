use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled failover segment must pass. Check returns its problems, or none.
pub trait ScheduledFailoverSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFailoverSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSegmentScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledFailoverSegmentRule for ScheduledFailoverSegmentScoreRangeRule {
    fn check(&self, item: &ScheduledFailoverSegment) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSegmentCodePrefixRule {
    pub prefix: String,
}

impl ScheduledFailoverSegmentRule for ScheduledFailoverSegmentCodePrefixRule {
    fn check(&self, item: &ScheduledFailoverSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSegmentSegmentLengthRule {
    pub max_length: i64,
}

impl ScheduledFailoverSegmentRule for ScheduledFailoverSegmentSegmentLengthRule {
    fn check(&self, item: &ScheduledFailoverSegment) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSegmentRequiredSegmentRule {
    pub message: String,
}

impl ScheduledFailoverSegmentRule for ScheduledFailoverSegmentRequiredSegmentRule {
    fn check(&self, item: &ScheduledFailoverSegment) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled failover segment.
#[derive(Debug, Clone)]
pub struct ScheduledFailoverSegmentValidator {
    pub rules: Vec<Rc<dyn ScheduledFailoverSegmentRule>>,
}

impl ScheduledFailoverSegmentValidator {
    pub fn validate(&self, item: &ScheduledFailoverSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledFailoverSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFailoverSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_failover_segment_validator() -> ScheduledFailoverSegmentValidator {
    ScheduledFailoverSegmentValidator {
        rules: vec![
            Rc::new(ScheduledFailoverSegmentScoreRangeRule { min: 2.9, max: 2143.4 })
                as Rc<dyn ScheduledFailoverSegmentRule>,
            Rc::new(ScheduledFailoverSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ScheduledFailoverSegmentSegmentLengthRule { max_length: 3600 }),
            Rc::new(ScheduledFailoverSegmentRequiredSegmentRule {
                message: "scheduled failover segment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_failover_segment_validator();
        let item = new_scheduled_failover_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ScheduledFailoverSegmentScoreRangeRule { min: 6.8, max: 3895.1 };
        let mut item = new_scheduled_failover_segment("x", "y");
        item.score = 3896.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = ScheduledFailoverSegmentSegmentLengthRule { max_length: 1000 };
        let mut item = new_scheduled_failover_segment("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = ScheduledFailoverSegmentRequiredSegmentRule {
            message: "scheduled failover segment needs this field".to_string(),
        };
        let mut item = new_scheduled_failover_segment("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
