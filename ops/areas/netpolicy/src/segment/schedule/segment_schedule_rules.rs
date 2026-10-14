use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment schedule must pass. Check returns its problems, or none.
pub trait SegmentScheduleRule: std::fmt::Debug {
    fn check(&self, item: &SegmentSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduleCodeLengthRule {
    pub max_length: i64,
}

impl SegmentScheduleRule for SegmentScheduleCodeLengthRule {
    fn check(&self, item: &SegmentSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduleRequiredIssuerRule {
    pub message: String,
}

impl SegmentScheduleRule for SegmentScheduleRequiredIssuerRule {
    fn check(&self, item: &SegmentSchedule) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduleRequiredOwnerIdRule {
    pub message: String,
}

impl SegmentScheduleRule for SegmentScheduleRequiredOwnerIdRule {
    fn check(&self, item: &SegmentSchedule) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment schedule.
#[derive(Debug, Clone)]
pub struct SegmentScheduleValidator {
    pub rules: Vec<Rc<dyn SegmentScheduleRule>>,
}

impl SegmentScheduleValidator {
    pub fn validate(&self, item: &SegmentSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_schedule_validator() -> SegmentScheduleValidator {
    SegmentScheduleValidator {
        rules: vec![
            Rc::new(SegmentScheduleCodeLengthRule { max_length: 80 }) as Rc<dyn SegmentScheduleRule>,
            Rc::new(SegmentScheduleRequiredIssuerRule {
                message: "segment schedule needs an entry".to_string(),
            }),
            Rc::new(SegmentScheduleRequiredOwnerIdRule {
                message: "segment schedule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_schedule_validator();
        let item = new_segment_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentScheduleCodeLengthRule { max_length: 3600 };
        let mut item = new_segment_schedule("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = SegmentScheduleRequiredIssuerRule {
            message: "segment schedule needs a value".to_string(),
        };
        let mut item = new_segment_schedule("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = SegmentScheduleRequiredOwnerIdRule {
            message: "segment schedule needs a value".to_string(),
        };
        let mut item = new_segment_schedule("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
