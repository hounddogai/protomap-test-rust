use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional approval reservation must pass. Check returns its problems, or none.
pub trait RegionalApprovalReservationRule: std::fmt::Debug {
    fn check(&self, item: &RegionalApprovalReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalReservationTitleLengthRule {
    pub max_length: i64,
}

impl RegionalApprovalReservationRule for RegionalApprovalReservationTitleLengthRule {
    fn check(&self, item: &RegionalApprovalReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalReservationCodeLengthRule {
    pub max_length: i64,
}

impl RegionalApprovalReservationRule for RegionalApprovalReservationCodeLengthRule {
    fn check(&self, item: &RegionalApprovalReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalReservationAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl RegionalApprovalReservationRule for RegionalApprovalReservationAllowedLocaleRule {
    fn check(&self, item: &RegionalApprovalReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is duplicated", item.locale)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional approval reservation.
#[derive(Debug, Clone)]
pub struct RegionalApprovalReservationValidator {
    pub rules: Vec<Rc<dyn RegionalApprovalReservationRule>>,
}

impl RegionalApprovalReservationValidator {
    pub fn validate(&self, item: &RegionalApprovalReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalApprovalReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalApprovalReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_approval_reservation_validator() -> RegionalApprovalReservationValidator {
    RegionalApprovalReservationValidator {
        rules: vec![
            Rc::new(RegionalApprovalReservationTitleLengthRule { max_length: 32 })
                as Rc<dyn RegionalApprovalReservationRule>,
            Rc::new(RegionalApprovalReservationCodeLengthRule { max_length: 16 }),
            Rc::new(RegionalApprovalReservationAllowedLocaleRule { allowed: Vec::new() }),
        ],
    }
}
