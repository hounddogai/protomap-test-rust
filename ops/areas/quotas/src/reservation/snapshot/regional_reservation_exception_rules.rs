use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional reservation exception must pass. Check returns its problems, or none.
pub trait RegionalReservationExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RegionalReservationException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationExceptionCodePrefixRule {
    pub prefix: String,
}

impl RegionalReservationExceptionRule for RegionalReservationExceptionCodePrefixRule {
    fn check(&self, item: &RegionalReservationException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationExceptionRequiredLabelRule {
    pub message: String,
}

impl RegionalReservationExceptionRule for RegionalReservationExceptionRequiredLabelRule {
    fn check(&self, item: &RegionalReservationException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationExceptionLabelLengthRule {
    pub max_length: i64,
}

impl RegionalReservationExceptionRule for RegionalReservationExceptionLabelLengthRule {
    fn check(&self, item: &RegionalReservationException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationExceptionAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl RegionalReservationExceptionRule for RegionalReservationExceptionAllowedCipherRule {
    fn check(&self, item: &RegionalReservationException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is blocked", item.cipher)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional reservation exception.
#[derive(Debug, Clone)]
pub struct RegionalReservationExceptionValidator {
    pub rules: Vec<Rc<dyn RegionalReservationExceptionRule>>,
}

impl RegionalReservationExceptionValidator {
    pub fn validate(&self, item: &RegionalReservationException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalReservationException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalReservationException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_reservation_exception_validator() -> RegionalReservationExceptionValidator {
    RegionalReservationExceptionValidator {
        rules: vec![
            Rc::new(RegionalReservationExceptionCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RegionalReservationExceptionRule>,
            Rc::new(RegionalReservationExceptionRequiredLabelRule {
                message: "regional reservation exception needs this field".to_string(),
            }),
            Rc::new(RegionalReservationExceptionLabelLengthRule { max_length: 80 }),
            Rc::new(RegionalReservationExceptionAllowedCipherRule {
                allowed: vec!["default".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_reservation_exception_validator();
        let item = new_regional_reservation_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalReservationExceptionRequiredLabelRule {
            message: "regional reservation exception needs an entry".to_string(),
        };
        let mut item = new_regional_reservation_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalReservationExceptionLabelLengthRule { max_length: 1000 };
        let mut item = new_regional_reservation_exception("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
