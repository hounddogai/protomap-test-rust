use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling verifier slot must pass. Check returns its problems, or none.
pub trait RollingVerifierSlotRule: std::fmt::Debug {
    fn check(&self, item: &RollingVerifierSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSlotScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingVerifierSlotRule for RollingVerifierSlotScoreRangeRule {
    fn check(&self, item: &RollingVerifierSlot) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSlotCodePrefixRule {
    pub prefix: String,
}

impl RollingVerifierSlotRule for RollingVerifierSlotCodePrefixRule {
    fn check(&self, item: &RollingVerifierSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSlotUnitLengthRule {
    pub max_length: i64,
}

impl RollingVerifierSlotRule for RollingVerifierSlotUnitLengthRule {
    fn check(&self, item: &RollingVerifierSlot) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling verifier slot.
#[derive(Debug, Clone)]
pub struct RollingVerifierSlotValidator {
    pub rules: Vec<Rc<dyn RollingVerifierSlotRule>>,
}

impl RollingVerifierSlotValidator {
    pub fn validate(&self, item: &RollingVerifierSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingVerifierSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingVerifierSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_verifier_slot_validator() -> RollingVerifierSlotValidator {
    RollingVerifierSlotValidator {
        rules: vec![
            Rc::new(RollingVerifierSlotScoreRangeRule { min: 3.2, max: 3344.0 }) as Rc<dyn RollingVerifierSlotRule>,
            Rc::new(RollingVerifierSlotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollingVerifierSlotUnitLengthRule { max_length: 3600 }),
        ],
    }
}
