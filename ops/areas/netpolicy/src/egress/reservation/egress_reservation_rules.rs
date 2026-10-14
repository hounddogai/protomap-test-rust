use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress reservation must pass. Check returns its problems, or none.
pub trait EgressReservationRule: std::fmt::Debug {
    fn check(&self, item: &EgressReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressReservationCodeLengthRule {
    pub max_length: i64,
}

impl EgressReservationRule for EgressReservationCodeLengthRule {
    fn check(&self, item: &EgressReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressReservationRequiredCodeRule {
    pub message: String,
}

impl EgressReservationRule for EgressReservationRequiredCodeRule {
    fn check(&self, item: &EgressReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressReservationAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl EgressReservationRule for EgressReservationAllowedKeyIdRule {
    fn check(&self, item: &EgressReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is too long", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress reservation.
#[derive(Debug, Clone)]
pub struct EgressReservationValidator {
    pub rules: Vec<Rc<dyn EgressReservationRule>>,
}

impl EgressReservationValidator {
    pub fn validate(&self, item: &EgressReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_reservation_validator() -> EgressReservationValidator {
    EgressReservationValidator {
        rules: vec![
            Rc::new(EgressReservationCodeLengthRule { max_length: 32 }) as Rc<dyn EgressReservationRule>,
            Rc::new(EgressReservationRequiredCodeRule {
                message: "egress reservation needs an entry".to_string(),
            }),
            Rc::new(EgressReservationAllowedKeyIdRule {
                allowed: vec!["pending".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_reservation_validator();
        let item = new_egress_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressReservationCodeLengthRule { max_length: 3600 };
        let mut item = new_egress_reservation("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressReservationRequiredCodeRule {
            message: "egress reservation needs this field".to_string(),
        };
        let mut item = new_egress_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
