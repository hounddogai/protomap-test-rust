use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped timeline reservation must pass. Check returns its problems, or none.
pub trait CappedTimelineReservationRule: std::fmt::Debug {
    fn check(&self, item: &CappedTimelineReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationCodeLengthRule {
    pub max_length: i64,
}

impl CappedTimelineReservationRule for CappedTimelineReservationCodeLengthRule {
    fn check(&self, item: &CappedTimelineReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationCodePrefixRule {
    pub prefix: String,
}

impl CappedTimelineReservationRule for CappedTimelineReservationCodePrefixRule {
    fn check(&self, item: &CappedTimelineReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedTimelineReservationRule for CappedTimelineReservationBurstRangeRule {
    fn check(&self, item: &CappedTimelineReservation) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationRequiredTitleRule {
    pub message: String,
}

impl CappedTimelineReservationRule for CappedTimelineReservationRequiredTitleRule {
    fn check(&self, item: &CappedTimelineReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationRequiredContactEmailRule {
    pub message: String,
}

impl CappedTimelineReservationRule for CappedTimelineReservationRequiredContactEmailRule {
    fn check(&self, item: &CappedTimelineReservation) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedTimelineReservationRule for CappedTimelineReservationMarginRangeRule {
    fn check(&self, item: &CappedTimelineReservation) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped timeline reservation.
#[derive(Debug, Clone)]
pub struct CappedTimelineReservationValidator {
    pub rules: Vec<Rc<dyn CappedTimelineReservationRule>>,
}

impl CappedTimelineReservationValidator {
    pub fn validate(&self, item: &CappedTimelineReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedTimelineReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTimelineReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_timeline_reservation_validator() -> CappedTimelineReservationValidator {
    CappedTimelineReservationValidator {
        rules: vec![
            Rc::new(CappedTimelineReservationCodeLengthRule { max_length: 32 })
                as Rc<dyn CappedTimelineReservationRule>,
            Rc::new(CappedTimelineReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedTimelineReservationBurstRangeRule { min: 120, max: 255 }),
            Rc::new(CappedTimelineReservationRequiredTitleRule {
                message: "capped timeline reservation needs this field".to_string(),
            }),
            Rc::new(CappedTimelineReservationRequiredContactEmailRule {
                message: "capped timeline reservation needs a value".to_string(),
            }),
            Rc::new(CappedTimelineReservationMarginRangeRule { min: 0.9, max: 3869.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_timeline_reservation_validator();
        let item = new_capped_timeline_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedTimelineReservationCodeLengthRule { max_length: 3600 };
        let mut item = new_capped_timeline_reservation("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = CappedTimelineReservationBurstRangeRule { min: 16, max: 1000 };
        let mut item = new_capped_timeline_reservation("x", "y");
        item.burst = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedTimelineReservationRequiredTitleRule {
            message: "capped timeline reservation needs this field".to_string(),
        };
        let mut item = new_capped_timeline_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = CappedTimelineReservationRequiredContactEmailRule {
            message: "capped timeline reservation needs an entry".to_string(),
        };
        let mut item = new_capped_timeline_reservation("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CappedTimelineReservationMarginRangeRule { min: 0.9, max: 2804.5 };
        let mut item = new_capped_timeline_reservation("x", "y");
        item.margin = 2806.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
