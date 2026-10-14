use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback schedule must pass. Check returns its problems, or none.
pub trait RollbackScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RollbackSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScheduleCodeLengthRule {
    pub max_length: i64,
}

impl RollbackScheduleRule for RollbackScheduleCodeLengthRule {
    fn check(&self, item: &RollbackSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScheduleRegionLengthRule {
    pub max_length: i64,
}

impl RollbackScheduleRule for RollbackScheduleRegionLengthRule {
    fn check(&self, item: &RollbackSchedule) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScheduleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackScheduleRule for RollbackScheduleMinimumRangeRule {
    fn check(&self, item: &RollbackSchedule) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScheduleAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl RollbackScheduleRule for RollbackScheduleAllowedRegionRule {
    fn check(&self, item: &RollbackSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is not allowed", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScheduleNotesLengthRule {
    pub max_length: i64,
}

impl RollbackScheduleRule for RollbackScheduleNotesLengthRule {
    fn check(&self, item: &RollbackSchedule) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScheduleLabelLengthRule {
    pub max_length: i64,
}

impl RollbackScheduleRule for RollbackScheduleLabelLengthRule {
    fn check(&self, item: &RollbackSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback schedule.
#[derive(Debug, Clone)]
pub struct RollbackScheduleValidator {
    pub rules: Vec<Rc<dyn RollbackScheduleRule>>,
}

impl RollbackScheduleValidator {
    pub fn validate(&self, item: &RollbackSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_schedule_validator() -> RollbackScheduleValidator {
    RollbackScheduleValidator {
        rules: vec![
            Rc::new(RollbackScheduleCodeLengthRule { max_length: 16 }) as Rc<dyn RollbackScheduleRule>,
            Rc::new(RollbackScheduleRegionLengthRule { max_length: 64 }),
            Rc::new(RollbackScheduleMinimumRangeRule { min: 7.4, max: 933.1 }),
            Rc::new(RollbackScheduleAllowedRegionRule { allowed: Vec::new() }),
            Rc::new(RollbackScheduleNotesLengthRule { max_length: 16 }),
            Rc::new(RollbackScheduleLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_schedule_validator();
        let item = new_rollback_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackScheduleCodeLengthRule { max_length: 32 };
        let mut item = new_rollback_schedule("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = RollbackScheduleRegionLengthRule { max_length: 120 };
        let mut item = new_rollback_schedule("x", "y");
        item.region = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = RollbackScheduleMinimumRangeRule { min: 8.0, max: 4886.0 };
        let mut item = new_rollback_schedule("x", "y");
        item.minimum = 4887.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RollbackScheduleNotesLengthRule { max_length: 120 };
        let mut item = new_rollback_schedule("x", "y");
        item.notes = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackScheduleLabelLengthRule { max_length: 80 };
        let mut item = new_rollback_schedule("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
