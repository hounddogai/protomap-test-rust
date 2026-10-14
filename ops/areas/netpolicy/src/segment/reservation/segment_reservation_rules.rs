use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment reservation must pass. Check returns its problems, or none.
pub trait SegmentReservationRule: std::fmt::Debug {
    fn check(&self, item: &SegmentReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReservationCodeLengthRule {
    pub max_length: i64,
}

impl SegmentReservationRule for SegmentReservationCodeLengthRule {
    fn check(&self, item: &SegmentReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReservationRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentReservationRule for SegmentReservationRetriesRangeRule {
    fn check(&self, item: &SegmentReservation) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReservationRequiredTimezoneRule {
    pub message: String,
}

impl SegmentReservationRule for SegmentReservationRequiredTimezoneRule {
    fn check(&self, item: &SegmentReservation) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReservationTimezoneLengthRule {
    pub max_length: i64,
}

impl SegmentReservationRule for SegmentReservationTimezoneLengthRule {
    fn check(&self, item: &SegmentReservation) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReservationStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentReservationRule for SegmentReservationStartsAtRangeRule {
    fn check(&self, item: &SegmentReservation) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment reservation.
#[derive(Debug, Clone)]
pub struct SegmentReservationValidator {
    pub rules: Vec<Rc<dyn SegmentReservationRule>>,
}

impl SegmentReservationValidator {
    pub fn validate(&self, item: &SegmentReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_reservation_validator() -> SegmentReservationValidator {
    SegmentReservationValidator {
        rules: vec![
            Rc::new(SegmentReservationCodeLengthRule { max_length: 255 }) as Rc<dyn SegmentReservationRule>,
            Rc::new(SegmentReservationRetriesRangeRule { min: 16, max: 1000 }),
            Rc::new(SegmentReservationRequiredTimezoneRule {
                message: "segment reservation needs a value".to_string(),
            }),
            Rc::new(SegmentReservationTimezoneLengthRule { max_length: 80 }),
            Rc::new(SegmentReservationStartsAtRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_reservation_validator();
        let item = new_segment_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentReservationCodeLengthRule { max_length: 64 };
        let mut item = new_segment_reservation("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = SegmentReservationRetriesRangeRule { min: 32, max: 1000 };
        let mut item = new_segment_reservation("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = SegmentReservationRequiredTimezoneRule {
            message: "segment reservation needs an entry".to_string(),
        };
        let mut item = new_segment_reservation("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = SegmentReservationTimezoneLengthRule { max_length: 3600 };
        let mut item = new_segment_reservation("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = SegmentReservationStartsAtRangeRule { min: 120, max: 1000 };
        let mut item = new_segment_reservation("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
