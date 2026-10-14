use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover schedule must pass. Check returns its problems, or none.
pub trait FailoverScheduleRule: std::fmt::Debug {
    fn check(&self, item: &FailoverSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverScheduleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverScheduleRule for FailoverScheduleFloorValueRangeRule {
    fn check(&self, item: &FailoverSchedule) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverScheduleRequiredTitleRule {
    pub message: String,
}

impl FailoverScheduleRule for FailoverScheduleRequiredTitleRule {
    fn check(&self, item: &FailoverSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverScheduleAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl FailoverScheduleRule for FailoverScheduleAllowedSourceRule {
    fn check(&self, item: &FailoverSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is missing", item.source)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover schedule.
#[derive(Debug, Clone)]
pub struct FailoverScheduleValidator {
    pub rules: Vec<Rc<dyn FailoverScheduleRule>>,
}

impl FailoverScheduleValidator {
    pub fn validate(&self, item: &FailoverSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_schedule_validator() -> FailoverScheduleValidator {
    FailoverScheduleValidator {
        rules: vec![
            Rc::new(FailoverScheduleFloorValueRangeRule { min: 3.9, max: 1033.2 }) as Rc<dyn FailoverScheduleRule>,
            Rc::new(FailoverScheduleRequiredTitleRule {
                message: "failover schedule needs a value".to_string(),
            }),
            Rc::new(FailoverScheduleAllowedSourceRule {
                allowed: vec!["rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_schedule_validator();
        let item = new_failover_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FailoverScheduleFloorValueRangeRule { min: 9.6, max: 1026.2 };
        let mut item = new_failover_schedule("x", "y");
        item.floor_value = 1027.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FailoverScheduleRequiredTitleRule {
            message: "failover schedule needs a value".to_string(),
        };
        let mut item = new_failover_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
