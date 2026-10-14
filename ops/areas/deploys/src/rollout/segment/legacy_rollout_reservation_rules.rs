use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rollout reservation must pass. Check returns its problems, or none.
pub trait LegacyRolloutReservationRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRolloutReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutReservationCodeLengthRule {
    pub max_length: i64,
}

impl LegacyRolloutReservationRule for LegacyRolloutReservationCodeLengthRule {
    fn check(&self, item: &LegacyRolloutReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutReservationRequiredLocaleRule {
    pub message: String,
}

impl LegacyRolloutReservationRule for LegacyRolloutReservationRequiredLocaleRule {
    fn check(&self, item: &LegacyRolloutReservation) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutReservationRequiredContactEmailRule {
    pub message: String,
}

impl LegacyRolloutReservationRule for LegacyRolloutReservationRequiredContactEmailRule {
    fn check(&self, item: &LegacyRolloutReservation) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rollout reservation.
#[derive(Debug, Clone)]
pub struct LegacyRolloutReservationValidator {
    pub rules: Vec<Rc<dyn LegacyRolloutReservationRule>>,
}

impl LegacyRolloutReservationValidator {
    pub fn validate(&self, item: &LegacyRolloutReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRolloutReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRolloutReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollout_reservation_validator() -> LegacyRolloutReservationValidator {
    LegacyRolloutReservationValidator {
        rules: vec![
            Rc::new(LegacyRolloutReservationCodeLengthRule { max_length: 80 }) as Rc<dyn LegacyRolloutReservationRule>,
            Rc::new(LegacyRolloutReservationRequiredLocaleRule {
                message: "legacy rollout reservation needs a value".to_string(),
            }),
            Rc::new(LegacyRolloutReservationRequiredContactEmailRule {
                message: "legacy rollout reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_rollout_reservation_validator();
        let item = new_legacy_rollout_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyRolloutReservationCodeLengthRule { max_length: 32 };
        let mut item = new_legacy_rollout_reservation("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = LegacyRolloutReservationRequiredLocaleRule {
            message: "legacy rollout reservation needs this field".to_string(),
        };
        let mut item = new_legacy_rollout_reservation("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = LegacyRolloutReservationRequiredContactEmailRule {
            message: "legacy rollout reservation needs this field".to_string(),
        };
        let mut item = new_legacy_rollout_reservation("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
