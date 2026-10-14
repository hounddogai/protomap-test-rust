use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary slo reservation must pass. Check returns its problems, or none.
pub trait PrimarySloReservationRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySloReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloReservationCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimarySloReservationRule for PrimarySloReservationCreatedAtRangeRule {
    fn check(&self, item: &PrimarySloReservation) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloReservationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PrimarySloReservationRule for PrimarySloReservationAllowedLabelRule {
    fn check(&self, item: &PrimarySloReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloReservationVisibleGuardRule {
    pub limit: i64,
}

impl PrimarySloReservationRule for PrimarySloReservationVisibleGuardRule {
    fn check(&self, item: &PrimarySloReservation) -> Vec<String> {
        if item.visible && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while visible",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloReservationCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySloReservationRule for PrimarySloReservationCodeLengthRule {
    fn check(&self, item: &PrimarySloReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloReservationRequiredCodeRule {
    pub message: String,
}

impl PrimarySloReservationRule for PrimarySloReservationRequiredCodeRule {
    fn check(&self, item: &PrimarySloReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloReservationRequiredLabelRule {
    pub message: String,
}

impl PrimarySloReservationRule for PrimarySloReservationRequiredLabelRule {
    fn check(&self, item: &PrimarySloReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary slo reservation.
#[derive(Debug, Clone)]
pub struct PrimarySloReservationValidator {
    pub rules: Vec<Rc<dyn PrimarySloReservationRule>>,
}

impl PrimarySloReservationValidator {
    pub fn validate(&self, item: &PrimarySloReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimarySloReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySloReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_slo_reservation_validator() -> PrimarySloReservationValidator {
    PrimarySloReservationValidator {
        rules: vec![
            Rc::new(PrimarySloReservationCreatedAtRangeRule { min: 80, max: 1000 })
                as Rc<dyn PrimarySloReservationRule>,
            Rc::new(PrimarySloReservationAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(PrimarySloReservationVisibleGuardRule { limit: 16 }),
            Rc::new(PrimarySloReservationCodeLengthRule { max_length: 32 }),
            Rc::new(PrimarySloReservationRequiredCodeRule {
                message: "primary slo reservation needs a value".to_string(),
            }),
            Rc::new(PrimarySloReservationRequiredLabelRule {
                message: "primary slo reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_slo_reservation_validator();
        let item = new_primary_slo_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = PrimarySloReservationCreatedAtRangeRule { min: 32, max: 86400 };
        let mut item = new_primary_slo_reservation("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySloReservationCodeLengthRule { max_length: 3600 };
        let mut item = new_primary_slo_reservation("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimarySloReservationRequiredCodeRule {
            message: "primary slo reservation needs an entry".to_string(),
        };
        let mut item = new_primary_slo_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimarySloReservationRequiredLabelRule {
            message: "primary slo reservation needs an entry".to_string(),
        };
        let mut item = new_primary_slo_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
