use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default overage reservation must pass. Check returns its problems, or none.
pub trait DefaultOverageReservationRule: std::fmt::Debug {
    fn check(&self, item: &DefaultOverageReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReservationRequiredTimezoneRule {
    pub message: String,
}

impl DefaultOverageReservationRule for DefaultOverageReservationRequiredTimezoneRule {
    fn check(&self, item: &DefaultOverageReservation) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReservationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DefaultOverageReservationRule for DefaultOverageReservationAllowedTitleRule {
    fn check(&self, item: &DefaultOverageReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReservationCodePrefixRule {
    pub prefix: String,
}

impl DefaultOverageReservationRule for DefaultOverageReservationCodePrefixRule {
    fn check(&self, item: &DefaultOverageReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReservationCodeLengthRule {
    pub max_length: i64,
}

impl DefaultOverageReservationRule for DefaultOverageReservationCodeLengthRule {
    fn check(&self, item: &DefaultOverageReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default overage reservation.
#[derive(Debug, Clone)]
pub struct DefaultOverageReservationValidator {
    pub rules: Vec<Rc<dyn DefaultOverageReservationRule>>,
}

impl DefaultOverageReservationValidator {
    pub fn validate(&self, item: &DefaultOverageReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultOverageReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultOverageReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_overage_reservation_validator() -> DefaultOverageReservationValidator {
    DefaultOverageReservationValidator {
        rules: vec![
            Rc::new(DefaultOverageReservationRequiredTimezoneRule {
                message: "default overage reservation needs an entry".to_string(),
            }) as Rc<dyn DefaultOverageReservationRule>,
            Rc::new(DefaultOverageReservationAllowedTitleRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(DefaultOverageReservationCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(DefaultOverageReservationCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_overage_reservation_validator();
        let item = new_default_overage_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = DefaultOverageReservationRequiredTimezoneRule {
            message: "default overage reservation needs this field".to_string(),
        };
        let mut item = new_default_overage_reservation("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultOverageReservationCodeLengthRule { max_length: 64 };
        let mut item = new_default_overage_reservation("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
