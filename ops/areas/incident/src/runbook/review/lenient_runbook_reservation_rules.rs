use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient runbook reservation must pass. Check returns its problems, or none.
pub trait LenientRunbookReservationRule: std::fmt::Debug {
    fn check(&self, item: &LenientRunbookReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookReservationTitleLengthRule {
    pub max_length: i64,
}

impl LenientRunbookReservationRule for LenientRunbookReservationTitleLengthRule {
    fn check(&self, item: &LenientRunbookReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookReservationRequiredCodeRule {
    pub message: String,
}

impl LenientRunbookReservationRule for LenientRunbookReservationRequiredCodeRule {
    fn check(&self, item: &LenientRunbookReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookReservationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRunbookReservationRule for LenientRunbookReservationMarginRangeRule {
    fn check(&self, item: &LenientRunbookReservation) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookReservationRequiredTitleRule {
    pub message: String,
}

impl LenientRunbookReservationRule for LenientRunbookReservationRequiredTitleRule {
    fn check(&self, item: &LenientRunbookReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookReservationOwnerIdLengthRule {
    pub max_length: i64,
}

impl LenientRunbookReservationRule for LenientRunbookReservationOwnerIdLengthRule {
    fn check(&self, item: &LenientRunbookReservation) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookReservationCodeLengthRule {
    pub max_length: i64,
}

impl LenientRunbookReservationRule for LenientRunbookReservationCodeLengthRule {
    fn check(&self, item: &LenientRunbookReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient runbook reservation.
#[derive(Debug, Clone)]
pub struct LenientRunbookReservationValidator {
    pub rules: Vec<Rc<dyn LenientRunbookReservationRule>>,
}

impl LenientRunbookReservationValidator {
    pub fn validate(&self, item: &LenientRunbookReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRunbookReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRunbookReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_runbook_reservation_validator() -> LenientRunbookReservationValidator {
    LenientRunbookReservationValidator {
        rules: vec![
            Rc::new(LenientRunbookReservationTitleLengthRule { max_length: 255 })
                as Rc<dyn LenientRunbookReservationRule>,
            Rc::new(LenientRunbookReservationRequiredCodeRule {
                message: "lenient runbook reservation needs a value".to_string(),
            }),
            Rc::new(LenientRunbookReservationMarginRangeRule { min: 3.4, max: 431.0 }),
            Rc::new(LenientRunbookReservationRequiredTitleRule {
                message: "lenient runbook reservation needs this field".to_string(),
            }),
            Rc::new(LenientRunbookReservationOwnerIdLengthRule { max_length: 16 }),
            Rc::new(LenientRunbookReservationCodeLengthRule { max_length: 80 }),
        ],
    }
}
