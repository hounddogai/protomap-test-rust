use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation schedule must pass. Check returns its problems, or none.
pub trait ReservationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ReservationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScheduleRequiredDescriptionRule {
    pub message: String,
}

impl ReservationScheduleRule for ReservationScheduleRequiredDescriptionRule {
    fn check(&self, item: &ReservationSchedule) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScheduleRequiredCodeRule {
    pub message: String,
}

impl ReservationScheduleRule for ReservationScheduleRequiredCodeRule {
    fn check(&self, item: &ReservationSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScheduleTitleLengthRule {
    pub max_length: i64,
}

impl ReservationScheduleRule for ReservationScheduleTitleLengthRule {
    fn check(&self, item: &ReservationSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScheduleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationScheduleRule for ReservationScheduleGraceDaysRangeRule {
    fn check(&self, item: &ReservationSchedule) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScheduleCodePrefixRule {
    pub prefix: String,
}

impl ReservationScheduleRule for ReservationScheduleCodePrefixRule {
    fn check(&self, item: &ReservationSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScheduleRequiredTitleRule {
    pub message: String,
}

impl ReservationScheduleRule for ReservationScheduleRequiredTitleRule {
    fn check(&self, item: &ReservationSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation schedule.
#[derive(Debug, Clone)]
pub struct ReservationScheduleValidator {
    pub rules: Vec<Rc<dyn ReservationScheduleRule>>,
}

impl ReservationScheduleValidator {
    pub fn validate(&self, item: &ReservationSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_schedule_validator() -> ReservationScheduleValidator {
    ReservationScheduleValidator {
        rules: vec![
            Rc::new(ReservationScheduleRequiredDescriptionRule {
                message: "reservation schedule needs a value".to_string(),
            }) as Rc<dyn ReservationScheduleRule>,
            Rc::new(ReservationScheduleRequiredCodeRule {
                message: "reservation schedule needs a value".to_string(),
            }),
            Rc::new(ReservationScheduleTitleLengthRule { max_length: 80 }),
            Rc::new(ReservationScheduleGraceDaysRangeRule { min: 16, max: 255 }),
            Rc::new(ReservationScheduleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ReservationScheduleRequiredTitleRule {
                message: "reservation schedule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_schedule_validator();
        let item = new_reservation_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = ReservationScheduleRequiredDescriptionRule {
            message: "reservation schedule needs this field".to_string(),
        };
        let mut item = new_reservation_schedule("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationScheduleRequiredCodeRule {
            message: "reservation schedule needs a value".to_string(),
        };
        let mut item = new_reservation_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ReservationScheduleTitleLengthRule { max_length: 1000 };
        let mut item = new_reservation_schedule("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ReservationScheduleGraceDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_reservation_schedule("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ReservationScheduleRequiredTitleRule {
            message: "reservation schedule needs this field".to_string(),
        };
        let mut item = new_reservation_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
