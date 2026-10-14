use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient rule reservation must pass. Check returns its problems, or none.
pub trait LenientRuleReservationRule: std::fmt::Debug {
    fn check(&self, item: &LenientRuleReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservationRequiredTitleRule {
    pub message: String,
}

impl LenientRuleReservationRule for LenientRuleReservationRequiredTitleRule {
    fn check(&self, item: &LenientRuleReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservationMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRuleReservationRule for LenientRuleReservationMaximumRangeRule {
    fn check(&self, item: &LenientRuleReservation) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservationQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRuleReservationRule for LenientRuleReservationQuantityRangeRule {
    fn check(&self, item: &LenientRuleReservation) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservationCodeLengthRule {
    pub max_length: i64,
}

impl LenientRuleReservationRule for LenientRuleReservationCodeLengthRule {
    fn check(&self, item: &LenientRuleReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LenientRuleReservationRule for LenientRuleReservationAllowedTitleRule {
    fn check(&self, item: &LenientRuleReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservationTitleLengthRule {
    pub max_length: i64,
}

impl LenientRuleReservationRule for LenientRuleReservationTitleLengthRule {
    fn check(&self, item: &LenientRuleReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient rule reservation.
#[derive(Debug, Clone)]
pub struct LenientRuleReservationValidator {
    pub rules: Vec<Rc<dyn LenientRuleReservationRule>>,
}

impl LenientRuleReservationValidator {
    pub fn validate(&self, item: &LenientRuleReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRuleReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRuleReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_rule_reservation_validator() -> LenientRuleReservationValidator {
    LenientRuleReservationValidator {
        rules: vec![
            Rc::new(LenientRuleReservationRequiredTitleRule {
                message: "lenient rule reservation needs an entry".to_string(),
            }) as Rc<dyn LenientRuleReservationRule>,
            Rc::new(LenientRuleReservationMaximumRangeRule { min: 5.8, max: 2817.9 }),
            Rc::new(LenientRuleReservationQuantityRangeRule { min: 16, max: 1000 }),
            Rc::new(LenientRuleReservationCodeLengthRule { max_length: 3600 }),
            Rc::new(LenientRuleReservationAllowedTitleRule {
                allowed: vec!["primary".to_string(), "batched".to_string(), "strict".to_string()],
            }),
            Rc::new(LenientRuleReservationTitleLengthRule { max_length: 3600 }),
        ],
    }
}
