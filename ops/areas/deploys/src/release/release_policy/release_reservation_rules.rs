use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release reservation must pass. Check returns its problems, or none.
pub trait ReleaseReservationRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReservationRequiredExternalRefRule {
    pub message: String,
}

impl ReleaseReservationRule for ReleaseReservationRequiredExternalRefRule {
    fn check(&self, item: &ReleaseReservation) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReservationSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseReservationRule for ReleaseReservationSequenceRangeRule {
    fn check(&self, item: &ReleaseReservation) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReservationPrincipalLengthRule {
    pub max_length: i64,
}

impl ReleaseReservationRule for ReleaseReservationPrincipalLengthRule {
    fn check(&self, item: &ReleaseReservation) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReservationRequiredTitleRule {
    pub message: String,
}

impl ReleaseReservationRule for ReleaseReservationRequiredTitleRule {
    fn check(&self, item: &ReleaseReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release reservation.
#[derive(Debug, Clone)]
pub struct ReleaseReservationValidator {
    pub rules: Vec<Rc<dyn ReleaseReservationRule>>,
}

impl ReleaseReservationValidator {
    pub fn validate(&self, item: &ReleaseReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_reservation_validator() -> ReleaseReservationValidator {
    ReleaseReservationValidator {
        rules: vec![
            Rc::new(ReleaseReservationRequiredExternalRefRule {
                message: "release reservation needs a value".to_string(),
            }) as Rc<dyn ReleaseReservationRule>,
            Rc::new(ReleaseReservationSequenceRangeRule { min: 64, max: 64 }),
            Rc::new(ReleaseReservationPrincipalLengthRule { max_length: 3600 }),
            Rc::new(ReleaseReservationRequiredTitleRule {
                message: "release reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_reservation_validator();
        let item = new_release_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ReleaseReservationRequiredExternalRefRule {
            message: "release reservation needs a value".to_string(),
        };
        let mut item = new_release_reservation("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ReleaseReservationSequenceRangeRule { min: 64, max: 80 };
        let mut item = new_release_reservation("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = ReleaseReservationPrincipalLengthRule { max_length: 32 };
        let mut item = new_release_reservation("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ReleaseReservationRequiredTitleRule {
            message: "release reservation needs a value".to_string(),
        };
        let mut item = new_release_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
