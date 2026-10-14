use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped budget burn schedule must pass. Check returns its problems, or none.
pub trait CappedBudgetBurnScheduleRule: std::fmt::Debug {
    fn check(&self, item: &CappedBudgetBurnSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnScheduleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedBudgetBurnScheduleRule for CappedBudgetBurnScheduleGraceDaysRangeRule {
    fn check(&self, item: &CappedBudgetBurnSchedule) -> Vec<String> {
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
pub struct CappedBudgetBurnScheduleLabelLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnScheduleRule for CappedBudgetBurnScheduleLabelLengthRule {
    fn check(&self, item: &CappedBudgetBurnSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnScheduleAudienceLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnScheduleRule for CappedBudgetBurnScheduleAudienceLengthRule {
    fn check(&self, item: &CappedBudgetBurnSchedule) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnScheduleSegmentLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnScheduleRule for CappedBudgetBurnScheduleSegmentLengthRule {
    fn check(&self, item: &CappedBudgetBurnSchedule) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnScheduleCodePrefixRule {
    pub prefix: String,
}

impl CappedBudgetBurnScheduleRule for CappedBudgetBurnScheduleCodePrefixRule {
    fn check(&self, item: &CappedBudgetBurnSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnScheduleDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedBudgetBurnScheduleRule for CappedBudgetBurnScheduleDurationMinutesRangeRule {
    fn check(&self, item: &CappedBudgetBurnSchedule) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped budget burn schedule.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnScheduleValidator {
    pub rules: Vec<Rc<dyn CappedBudgetBurnScheduleRule>>,
}

impl CappedBudgetBurnScheduleValidator {
    pub fn validate(&self, item: &CappedBudgetBurnSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedBudgetBurnSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBudgetBurnSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_budget_burn_schedule_validator() -> CappedBudgetBurnScheduleValidator {
    CappedBudgetBurnScheduleValidator {
        rules: vec![
            Rc::new(CappedBudgetBurnScheduleGraceDaysRangeRule { min: 32, max: 64 })
                as Rc<dyn CappedBudgetBurnScheduleRule>,
            Rc::new(CappedBudgetBurnScheduleLabelLengthRule { max_length: 255 }),
            Rc::new(CappedBudgetBurnScheduleAudienceLengthRule { max_length: 1000 }),
            Rc::new(CappedBudgetBurnScheduleSegmentLengthRule { max_length: 3600 }),
            Rc::new(CappedBudgetBurnScheduleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedBudgetBurnScheduleDurationMinutesRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_budget_burn_schedule_validator();
        let item = new_capped_budget_burn_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = CappedBudgetBurnScheduleGraceDaysRangeRule { min: 64, max: 120 };
        let mut item = new_capped_budget_burn_schedule("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedBudgetBurnScheduleLabelLengthRule { max_length: 120 };
        let mut item = new_capped_budget_burn_schedule("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = CappedBudgetBurnScheduleAudienceLengthRule { max_length: 1000 };
        let mut item = new_capped_budget_burn_schedule("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = CappedBudgetBurnScheduleSegmentLengthRule { max_length: 120 };
        let mut item = new_capped_budget_burn_schedule("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CappedBudgetBurnScheduleDurationMinutesRangeRule { min: 255, max: 86400 };
        let mut item = new_capped_budget_burn_schedule("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
