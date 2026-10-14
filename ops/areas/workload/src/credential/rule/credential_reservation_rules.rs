use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential reservation must pass. Check returns its problems, or none.
pub trait CredentialReservationRule: std::fmt::Debug {
    fn check(&self, item: &CredentialReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReservationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialReservationRule for CredentialReservationGraceDaysRangeRule {
    fn check(&self, item: &CredentialReservation) -> Vec<String> {
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
pub struct CredentialReservationNameLengthRule {
    pub max_length: i64,
}

impl CredentialReservationRule for CredentialReservationNameLengthRule {
    fn check(&self, item: &CredentialReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReservationRequiredNameRule {
    pub message: String,
}

impl CredentialReservationRule for CredentialReservationRequiredNameRule {
    fn check(&self, item: &CredentialReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential reservation.
#[derive(Debug, Clone)]
pub struct CredentialReservationValidator {
    pub rules: Vec<Rc<dyn CredentialReservationRule>>,
}

impl CredentialReservationValidator {
    pub fn validate(&self, item: &CredentialReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_reservation_validator() -> CredentialReservationValidator {
    CredentialReservationValidator {
        rules: vec![
            Rc::new(CredentialReservationGraceDaysRangeRule { min: 255, max: 1000 })
                as Rc<dyn CredentialReservationRule>,
            Rc::new(CredentialReservationNameLengthRule { max_length: 80 }),
            Rc::new(CredentialReservationRequiredNameRule {
                message: "credential reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_reservation_validator();
        let item = new_credential_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = CredentialReservationGraceDaysRangeRule { min: 16, max: 32 };
        let mut item = new_credential_reservation("x", "y");
        item.grace_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CredentialReservationNameLengthRule { max_length: 16 };
        let mut item = new_credential_reservation("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CredentialReservationRequiredNameRule {
            message: "credential reservation needs a value".to_string(),
        };
        let mut item = new_credential_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
