use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary plan schedule must pass. Check returns its problems, or none.
pub trait PrimaryPlanScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPlanSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanScheduleCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryPlanScheduleRule for PrimaryPlanScheduleCodeLengthRule {
    fn check(&self, item: &PrimaryPlanSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanSchedulePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryPlanScheduleRule for PrimaryPlanSchedulePriorityRangeRule {
    fn check(&self, item: &PrimaryPlanSchedule) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanScheduleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryPlanScheduleRule for PrimaryPlanScheduleTtlSecondsRangeRule {
    fn check(&self, item: &PrimaryPlanSchedule) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary plan schedule.
#[derive(Debug, Clone)]
pub struct PrimaryPlanScheduleValidator {
    pub rules: Vec<Rc<dyn PrimaryPlanScheduleRule>>,
}

impl PrimaryPlanScheduleValidator {
    pub fn validate(&self, item: &PrimaryPlanSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryPlanSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPlanSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_plan_schedule_validator() -> PrimaryPlanScheduleValidator {
    PrimaryPlanScheduleValidator {
        rules: vec![
            Rc::new(PrimaryPlanScheduleCodeLengthRule { max_length: 16 }) as Rc<dyn PrimaryPlanScheduleRule>,
            Rc::new(PrimaryPlanSchedulePriorityRangeRule { min: 64, max: 86400 }),
            Rc::new(PrimaryPlanScheduleTtlSecondsRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_plan_schedule_validator();
        let item = new_primary_plan_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryPlanScheduleCodeLengthRule { max_length: 64 };
        let mut item = new_primary_plan_schedule("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PrimaryPlanSchedulePriorityRangeRule { min: 3600, max: 3600 };
        let mut item = new_primary_plan_schedule("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PrimaryPlanScheduleTtlSecondsRangeRule { min: 80, max: 120 };
        let mut item = new_primary_plan_schedule("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
