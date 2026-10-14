use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient reservation allocation must pass. Check returns its problems, or none.
pub trait LenientReservationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &LenientReservationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAllocationActiveGuardRule {
    pub limit: f64,
}

impl LenientReservationAllocationRule for LenientReservationAllocationActiveGuardRule {
    fn check(&self, item: &LenientReservationAllocation) -> Vec<String> {
        if item.active && (item.maximum > self.limit) {
            return vec![format!("maximum {} exceeds {} while active", item.maximum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAllocationRequiredCodeRule {
    pub message: String,
}

impl LenientReservationAllocationRule for LenientReservationAllocationRequiredCodeRule {
    fn check(&self, item: &LenientReservationAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAllocationRequiredCurrencyRule {
    pub message: String,
}

impl LenientReservationAllocationRule for LenientReservationAllocationRequiredCurrencyRule {
    fn check(&self, item: &LenientReservationAllocation) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient reservation allocation.
#[derive(Debug, Clone)]
pub struct LenientReservationAllocationValidator {
    pub rules: Vec<Rc<dyn LenientReservationAllocationRule>>,
}

impl LenientReservationAllocationValidator {
    pub fn validate(&self, item: &LenientReservationAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientReservationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientReservationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_reservation_allocation_validator() -> LenientReservationAllocationValidator {
    LenientReservationAllocationValidator {
        rules: vec![
            Rc::new(LenientReservationAllocationActiveGuardRule { limit: 7.6 })
                as Rc<dyn LenientReservationAllocationRule>,
            Rc::new(LenientReservationAllocationRequiredCodeRule {
                message: "lenient reservation allocation needs a value".to_string(),
            }),
            Rc::new(LenientReservationAllocationRequiredCurrencyRule {
                message: "lenient reservation allocation needs an entry".to_string(),
            }),
        ],
    }
}
