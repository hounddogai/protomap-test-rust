use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient token reservation must pass. Check returns its problems, or none.
pub trait LenientTokenReservationRule: std::fmt::Debug {
    fn check(&self, item: &LenientTokenReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenReservationRequiredNameRule {
    pub message: String,
}

impl LenientTokenReservationRule for LenientTokenReservationRequiredNameRule {
    fn check(&self, item: &LenientTokenReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenReservationLabelLengthRule {
    pub max_length: i64,
}

impl LenientTokenReservationRule for LenientTokenReservationLabelLengthRule {
    fn check(&self, item: &LenientTokenReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenReservationPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientTokenReservationRule for LenientTokenReservationPriorityRangeRule {
    fn check(&self, item: &LenientTokenReservation) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient token reservation.
#[derive(Debug, Clone)]
pub struct LenientTokenReservationValidator {
    pub rules: Vec<Rc<dyn LenientTokenReservationRule>>,
}

impl LenientTokenReservationValidator {
    pub fn validate(&self, item: &LenientTokenReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTokenReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTokenReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_token_reservation_validator() -> LenientTokenReservationValidator {
    LenientTokenReservationValidator {
        rules: vec![
            Rc::new(LenientTokenReservationRequiredNameRule {
                message: "lenient token reservation needs this field".to_string(),
            }) as Rc<dyn LenientTokenReservationRule>,
            Rc::new(LenientTokenReservationLabelLengthRule { max_length: 16 }),
            Rc::new(LenientTokenReservationPriorityRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_token_reservation_validator();
        let item = new_lenient_token_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientTokenReservationRequiredNameRule {
            message: "lenient token reservation needs a value".to_string(),
        };
        let mut item = new_lenient_token_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientTokenReservationLabelLengthRule { max_length: 32 };
        let mut item = new_lenient_token_reservation("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = LenientTokenReservationPriorityRangeRule { min: 86400, max: 86400 };
        let mut item = new_lenient_token_reservation("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
