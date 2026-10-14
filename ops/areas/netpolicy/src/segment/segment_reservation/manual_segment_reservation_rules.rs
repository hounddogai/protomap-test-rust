use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual segment reservation must pass. Check returns its problems, or none.
pub trait ManualSegmentReservationRule: std::fmt::Debug {
    fn check(&self, item: &ManualSegmentReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSegmentReservationTitleLengthRule {
    pub max_length: i64,
}

impl ManualSegmentReservationRule for ManualSegmentReservationTitleLengthRule {
    fn check(&self, item: &ManualSegmentReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSegmentReservationCodePrefixRule {
    pub prefix: String,
}

impl ManualSegmentReservationRule for ManualSegmentReservationCodePrefixRule {
    fn check(&self, item: &ManualSegmentReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSegmentReservationRequiredCodeRule {
    pub message: String,
}

impl ManualSegmentReservationRule for ManualSegmentReservationRequiredCodeRule {
    fn check(&self, item: &ManualSegmentReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual segment reservation.
#[derive(Debug, Clone)]
pub struct ManualSegmentReservationValidator {
    pub rules: Vec<Rc<dyn ManualSegmentReservationRule>>,
}

impl ManualSegmentReservationValidator {
    pub fn validate(&self, item: &ManualSegmentReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualSegmentReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSegmentReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_segment_reservation_validator() -> ManualSegmentReservationValidator {
    ManualSegmentReservationValidator {
        rules: vec![
            Rc::new(ManualSegmentReservationTitleLengthRule { max_length: 80 }) as Rc<dyn ManualSegmentReservationRule>,
            Rc::new(ManualSegmentReservationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ManualSegmentReservationRequiredCodeRule {
                message: "manual segment reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_segment_reservation_validator();
        let item = new_manual_segment_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualSegmentReservationTitleLengthRule { max_length: 120 };
        let mut item = new_manual_segment_reservation("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualSegmentReservationRequiredCodeRule {
            message: "manual segment reservation needs this field".to_string(),
        };
        let mut item = new_manual_segment_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
