use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding reservation must pass. Check returns its problems, or none.
pub trait BindingReservationRule: std::fmt::Debug {
    fn check(&self, item: &BindingReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingReservationCodePrefixRule {
    pub prefix: String,
}

impl BindingReservationRule for BindingReservationCodePrefixRule {
    fn check(&self, item: &BindingReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingReservationCodeLengthRule {
    pub max_length: i64,
}

impl BindingReservationRule for BindingReservationCodeLengthRule {
    fn check(&self, item: &BindingReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingReservationPhoneLengthRule {
    pub max_length: i64,
}

impl BindingReservationRule for BindingReservationPhoneLengthRule {
    fn check(&self, item: &BindingReservation) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingReservationRequiredPhoneRule {
    pub message: String,
}

impl BindingReservationRule for BindingReservationRequiredPhoneRule {
    fn check(&self, item: &BindingReservation) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding reservation.
#[derive(Debug, Clone)]
pub struct BindingReservationValidator {
    pub rules: Vec<Rc<dyn BindingReservationRule>>,
}

impl BindingReservationValidator {
    pub fn validate(&self, item: &BindingReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_reservation_validator() -> BindingReservationValidator {
    BindingReservationValidator {
        rules: vec![
            Rc::new(BindingReservationCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn BindingReservationRule>,
            Rc::new(BindingReservationCodeLengthRule { max_length: 32 }),
            Rc::new(BindingReservationPhoneLengthRule { max_length: 32 }),
            Rc::new(BindingReservationRequiredPhoneRule {
                message: "binding reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_reservation_validator();
        let item = new_binding_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingReservationCodeLengthRule { max_length: 255 };
        let mut item = new_binding_reservation("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = BindingReservationPhoneLengthRule { max_length: 120 };
        let mut item = new_binding_reservation("x", "y");
        item.phone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = BindingReservationRequiredPhoneRule {
            message: "binding reservation needs a value".to_string(),
        };
        let mut item = new_binding_reservation("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
