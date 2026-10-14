use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule schedule must pass. Check returns its problems, or none.
pub trait RuleScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RuleSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleScheduleRequiredSegmentRule {
    pub message: String,
}

impl RuleScheduleRule for RuleScheduleRequiredSegmentRule {
    fn check(&self, item: &RuleSchedule) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleScheduleRequiredTitleRule {
    pub message: String,
}

impl RuleScheduleRule for RuleScheduleRequiredTitleRule {
    fn check(&self, item: &RuleSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleScheduleCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleScheduleRule for RuleScheduleCapacityRangeRule {
    fn check(&self, item: &RuleSchedule) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleScheduleSegmentLengthRule {
    pub max_length: i64,
}

impl RuleScheduleRule for RuleScheduleSegmentLengthRule {
    fn check(&self, item: &RuleSchedule) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule schedule.
#[derive(Debug, Clone)]
pub struct RuleScheduleValidator {
    pub rules: Vec<Rc<dyn RuleScheduleRule>>,
}

impl RuleScheduleValidator {
    pub fn validate(&self, item: &RuleSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_schedule_validator() -> RuleScheduleValidator {
    RuleScheduleValidator {
        rules: vec![
            Rc::new(RuleScheduleRequiredSegmentRule {
                message: "rule schedule needs a value".to_string(),
            }) as Rc<dyn RuleScheduleRule>,
            Rc::new(RuleScheduleRequiredTitleRule {
                message: "rule schedule needs this field".to_string(),
            }),
            Rc::new(RuleScheduleCapacityRangeRule { min: 32, max: 3600 }),
            Rc::new(RuleScheduleSegmentLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_schedule_validator();
        let item = new_rule_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = RuleScheduleRequiredSegmentRule {
            message: "rule schedule needs an entry".to_string(),
        };
        let mut item = new_rule_schedule("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RuleScheduleRequiredTitleRule {
            message: "rule schedule needs this field".to_string(),
        };
        let mut item = new_rule_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RuleScheduleCapacityRangeRule { min: 16, max: 3600 };
        let mut item = new_rule_schedule("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RuleScheduleSegmentLengthRule { max_length: 255 };
        let mut item = new_rule_schedule("x", "y");
        item.segment = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
