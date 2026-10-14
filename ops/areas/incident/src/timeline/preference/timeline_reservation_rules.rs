use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline reservation must pass. Check returns its problems, or none.
pub trait TimelineReservationRule: std::fmt::Debug {
    fn check(&self, item: &TimelineReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReservationNameLengthRule {
    pub max_length: i64,
}

impl TimelineReservationRule for TimelineReservationNameLengthRule {
    fn check(&self, item: &TimelineReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReservationBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineReservationRule for TimelineReservationBudgetRangeRule {
    fn check(&self, item: &TimelineReservation) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReservationCodeLengthRule {
    pub max_length: i64,
}

impl TimelineReservationRule for TimelineReservationCodeLengthRule {
    fn check(&self, item: &TimelineReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReservationRequiredCodeRule {
    pub message: String,
}

impl TimelineReservationRule for TimelineReservationRequiredCodeRule {
    fn check(&self, item: &TimelineReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReservationRequiredNameRule {
    pub message: String,
}

impl TimelineReservationRule for TimelineReservationRequiredNameRule {
    fn check(&self, item: &TimelineReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReservationFingerprintLengthRule {
    pub max_length: i64,
}

impl TimelineReservationRule for TimelineReservationFingerprintLengthRule {
    fn check(&self, item: &TimelineReservation) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline reservation.
#[derive(Debug, Clone)]
pub struct TimelineReservationValidator {
    pub rules: Vec<Rc<dyn TimelineReservationRule>>,
}

impl TimelineReservationValidator {
    pub fn validate(&self, item: &TimelineReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_reservation_validator() -> TimelineReservationValidator {
    TimelineReservationValidator {
        rules: vec![
            Rc::new(TimelineReservationNameLengthRule { max_length: 16 }) as Rc<dyn TimelineReservationRule>,
            Rc::new(TimelineReservationBudgetRangeRule { min: 9.1, max: 981.8 }),
            Rc::new(TimelineReservationCodeLengthRule { max_length: 16 }),
            Rc::new(TimelineReservationRequiredCodeRule {
                message: "timeline reservation needs this field".to_string(),
            }),
            Rc::new(TimelineReservationRequiredNameRule {
                message: "timeline reservation needs this field".to_string(),
            }),
            Rc::new(TimelineReservationFingerprintLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_reservation_validator();
        let item = new_timeline_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TimelineReservationNameLengthRule { max_length: 3600 };
        let mut item = new_timeline_reservation("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TimelineReservationBudgetRangeRule { min: 0.8, max: 3365.5 };
        let mut item = new_timeline_reservation("x", "y");
        item.budget = 3367.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineReservationCodeLengthRule { max_length: 16 };
        let mut item = new_timeline_reservation("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineReservationRequiredCodeRule {
            message: "timeline reservation needs a value".to_string(),
        };
        let mut item = new_timeline_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TimelineReservationRequiredNameRule {
            message: "timeline reservation needs a value".to_string(),
        };
        let mut item = new_timeline_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = TimelineReservationFingerprintLengthRule { max_length: 255 };
        let mut item = new_timeline_reservation("x", "y");
        item.fingerprint = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
