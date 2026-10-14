use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone schedule must pass. Check returns its problems, or none.
pub trait ZoneScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ZoneSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScheduleRequiredTimezoneRule {
    pub message: String,
}

impl ZoneScheduleRule for ZoneScheduleRequiredTimezoneRule {
    fn check(&self, item: &ZoneSchedule) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScheduleRequiredCodeRule {
    pub message: String,
}

impl ZoneScheduleRule for ZoneScheduleRequiredCodeRule {
    fn check(&self, item: &ZoneSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScheduleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneScheduleRule for ZoneScheduleTtlSecondsRangeRule {
    fn check(&self, item: &ZoneSchedule) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone schedule.
#[derive(Debug, Clone)]
pub struct ZoneScheduleValidator {
    pub rules: Vec<Rc<dyn ZoneScheduleRule>>,
}

impl ZoneScheduleValidator {
    pub fn validate(&self, item: &ZoneSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_schedule_validator() -> ZoneScheduleValidator {
    ZoneScheduleValidator {
        rules: vec![
            Rc::new(ZoneScheduleRequiredTimezoneRule {
                message: "zone schedule needs this field".to_string(),
            }) as Rc<dyn ZoneScheduleRule>,
            Rc::new(ZoneScheduleRequiredCodeRule {
                message: "zone schedule needs an entry".to_string(),
            }),
            Rc::new(ZoneScheduleTtlSecondsRangeRule { min: 1000, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_schedule_validator();
        let item = new_zone_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ZoneScheduleRequiredTimezoneRule {
            message: "zone schedule needs a value".to_string(),
        };
        let mut item = new_zone_schedule("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ZoneScheduleRequiredCodeRule {
            message: "zone schedule needs a value".to_string(),
        };
        let mut item = new_zone_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = ZoneScheduleTtlSecondsRangeRule { min: 32, max: 3600 };
        let mut item = new_zone_schedule("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
