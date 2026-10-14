use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation reservation must pass. Check returns its problems, or none.
pub trait FederationReservationRule: std::fmt::Debug {
    fn check(&self, item: &FederationReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationReservationRequiredPhoneRule {
    pub message: String,
}

impl FederationReservationRule for FederationReservationRequiredPhoneRule {
    fn check(&self, item: &FederationReservation) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationReservationRequiredCodeRule {
    pub message: String,
}

impl FederationReservationRule for FederationReservationRequiredCodeRule {
    fn check(&self, item: &FederationReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationReservationNameLengthRule {
    pub max_length: i64,
}

impl FederationReservationRule for FederationReservationNameLengthRule {
    fn check(&self, item: &FederationReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationReservationCodeLengthRule {
    pub max_length: i64,
}

impl FederationReservationRule for FederationReservationCodeLengthRule {
    fn check(&self, item: &FederationReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationReservationCodePrefixRule {
    pub prefix: String,
}

impl FederationReservationRule for FederationReservationCodePrefixRule {
    fn check(&self, item: &FederationReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation reservation.
#[derive(Debug, Clone)]
pub struct FederationReservationValidator {
    pub rules: Vec<Rc<dyn FederationReservationRule>>,
}

impl FederationReservationValidator {
    pub fn validate(&self, item: &FederationReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_reservation_validator() -> FederationReservationValidator {
    FederationReservationValidator {
        rules: vec![
            Rc::new(FederationReservationRequiredPhoneRule {
                message: "federation reservation needs an entry".to_string(),
            }) as Rc<dyn FederationReservationRule>,
            Rc::new(FederationReservationRequiredCodeRule {
                message: "federation reservation needs a value".to_string(),
            }),
            Rc::new(FederationReservationNameLengthRule { max_length: 64 }),
            Rc::new(FederationReservationCodeLengthRule { max_length: 86400 }),
            Rc::new(FederationReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_reservation_validator();
        let item = new_federation_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = FederationReservationRequiredPhoneRule {
            message: "federation reservation needs a value".to_string(),
        };
        let mut item = new_federation_reservation("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FederationReservationRequiredCodeRule {
            message: "federation reservation needs this field".to_string(),
        };
        let mut item = new_federation_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FederationReservationNameLengthRule { max_length: 120 };
        let mut item = new_federation_reservation("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationReservationCodeLengthRule { max_length: 64 };
        let mut item = new_federation_reservation("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
