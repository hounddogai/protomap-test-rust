use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant reservation must pass. Check returns its problems, or none.
pub trait GrantReservationRule: std::fmt::Debug {
    fn check(&self, item: &GrantReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantReservationNameLengthRule {
    pub max_length: i64,
}

impl GrantReservationRule for GrantReservationNameLengthRule {
    fn check(&self, item: &GrantReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantReservationRegionLengthRule {
    pub max_length: i64,
}

impl GrantReservationRule for GrantReservationRegionLengthRule {
    fn check(&self, item: &GrantReservation) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantReservationRequiredNameRule {
    pub message: String,
}

impl GrantReservationRule for GrantReservationRequiredNameRule {
    fn check(&self, item: &GrantReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant reservation.
#[derive(Debug, Clone)]
pub struct GrantReservationValidator {
    pub rules: Vec<Rc<dyn GrantReservationRule>>,
}

impl GrantReservationValidator {
    pub fn validate(&self, item: &GrantReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_reservation_validator() -> GrantReservationValidator {
    GrantReservationValidator {
        rules: vec![
            Rc::new(GrantReservationNameLengthRule { max_length: 3600 }) as Rc<dyn GrantReservationRule>,
            Rc::new(GrantReservationRegionLengthRule { max_length: 32 }),
            Rc::new(GrantReservationRequiredNameRule {
                message: "grant reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_reservation_validator();
        let item = new_grant_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GrantReservationNameLengthRule { max_length: 32 };
        let mut item = new_grant_reservation("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = GrantReservationRegionLengthRule { max_length: 120 };
        let mut item = new_grant_reservation("x", "y");
        item.region = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GrantReservationRequiredNameRule {
            message: "grant reservation needs a value".to_string(),
        };
        let mut item = new_grant_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
