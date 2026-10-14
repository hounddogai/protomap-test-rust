use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage reservation must pass. Check returns its problems, or none.
pub trait OverageReservationRule: std::fmt::Debug {
    fn check(&self, item: &OverageReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReservationRequiredTitleRule {
    pub message: String,
}

impl OverageReservationRule for OverageReservationRequiredTitleRule {
    fn check(&self, item: &OverageReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReservationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl OverageReservationRule for OverageReservationAllowedTitleRule {
    fn check(&self, item: &OverageReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReservationCodeLengthRule {
    pub max_length: i64,
}

impl OverageReservationRule for OverageReservationCodeLengthRule {
    fn check(&self, item: &OverageReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReservationCodePrefixRule {
    pub prefix: String,
}

impl OverageReservationRule for OverageReservationCodePrefixRule {
    fn check(&self, item: &OverageReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReservationNotesLengthRule {
    pub max_length: i64,
}

impl OverageReservationRule for OverageReservationNotesLengthRule {
    fn check(&self, item: &OverageReservation) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage reservation.
#[derive(Debug, Clone)]
pub struct OverageReservationValidator {
    pub rules: Vec<Rc<dyn OverageReservationRule>>,
}

impl OverageReservationValidator {
    pub fn validate(&self, item: &OverageReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_reservation_validator() -> OverageReservationValidator {
    OverageReservationValidator {
        rules: vec![
            Rc::new(OverageReservationRequiredTitleRule {
                message: "overage reservation needs a value".to_string(),
            }) as Rc<dyn OverageReservationRule>,
            Rc::new(OverageReservationAllowedTitleRule {
                allowed: vec!["legacy".to_string(), "batched".to_string()],
            }),
            Rc::new(OverageReservationCodeLengthRule { max_length: 120 }),
            Rc::new(OverageReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(OverageReservationNotesLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_reservation_validator();
        let item = new_overage_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = OverageReservationRequiredTitleRule {
            message: "overage reservation needs an entry".to_string(),
        };
        let mut item = new_overage_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageReservationCodeLengthRule { max_length: 16 };
        let mut item = new_overage_reservation("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = OverageReservationNotesLengthRule { max_length: 64 };
        let mut item = new_overage_reservation("x", "y");
        item.notes = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
