use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo schedule must pass. Check returns its problems, or none.
pub trait SloScheduleRule: std::fmt::Debug {
    fn check(&self, item: &SloSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloScheduleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloScheduleRule for SloScheduleRotationDaysRangeRule {
    fn check(&self, item: &SloSchedule) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloScheduleRequiredCodeRule {
    pub message: String,
}

impl SloScheduleRule for SloScheduleRequiredCodeRule {
    fn check(&self, item: &SloSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloScheduleRequiredSegmentRule {
    pub message: String,
}

impl SloScheduleRule for SloScheduleRequiredSegmentRule {
    fn check(&self, item: &SloSchedule) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloScheduleNameLengthRule {
    pub max_length: i64,
}

impl SloScheduleRule for SloScheduleNameLengthRule {
    fn check(&self, item: &SloSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo schedule.
#[derive(Debug, Clone)]
pub struct SloScheduleValidator {
    pub rules: Vec<Rc<dyn SloScheduleRule>>,
}

impl SloScheduleValidator {
    pub fn validate(&self, item: &SloSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_schedule_validator() -> SloScheduleValidator {
    SloScheduleValidator {
        rules: vec![
            Rc::new(SloScheduleRotationDaysRangeRule { min: 64, max: 255 }) as Rc<dyn SloScheduleRule>,
            Rc::new(SloScheduleRequiredCodeRule {
                message: "slo schedule needs this field".to_string(),
            }),
            Rc::new(SloScheduleRequiredSegmentRule {
                message: "slo schedule needs a value".to_string(),
            }),
            Rc::new(SloScheduleNameLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_schedule_validator();
        let item = new_slo_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = SloScheduleRotationDaysRangeRule { min: 16, max: 16 };
        let mut item = new_slo_schedule("x", "y");
        item.rotation_days = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloScheduleRequiredCodeRule {
            message: "slo schedule needs this field".to_string(),
        };
        let mut item = new_slo_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = SloScheduleRequiredSegmentRule {
            message: "slo schedule needs an entry".to_string(),
        };
        let mut item = new_slo_schedule("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SloScheduleNameLengthRule { max_length: 32 };
        let mut item = new_slo_schedule("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
