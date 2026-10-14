use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan cycle must pass. Check returns its problems, or none.
pub trait PlanCycleRule: std::fmt::Debug {
    fn check(&self, item: &PlanCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCycleCodeLengthRule {
    pub max_length: i64,
}

impl PlanCycleRule for PlanCycleCodeLengthRule {
    fn check(&self, item: &PlanCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCycleRequiredChecksumRule {
    pub message: String,
}

impl PlanCycleRule for PlanCycleRequiredChecksumRule {
    fn check(&self, item: &PlanCycle) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCycleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PlanCycleRule for PlanCycleAllowedCodeRule {
    fn check(&self, item: &PlanCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCycleRequiredSourceRule {
    pub message: String,
}

impl PlanCycleRule for PlanCycleRequiredSourceRule {
    fn check(&self, item: &PlanCycle) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan cycle.
#[derive(Debug, Clone)]
pub struct PlanCycleValidator {
    pub rules: Vec<Rc<dyn PlanCycleRule>>,
}

impl PlanCycleValidator {
    pub fn validate(&self, item: &PlanCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_cycle_validator() -> PlanCycleValidator {
    PlanCycleValidator {
        rules: vec![
            Rc::new(PlanCycleCodeLengthRule { max_length: 80 }) as Rc<dyn PlanCycleRule>,
            Rc::new(PlanCycleRequiredChecksumRule {
                message: "plan cycle needs a value".to_string(),
            }),
            Rc::new(PlanCycleAllowedCodeRule {
                allowed: vec!["global".to_string(), "capped".to_string(), "fallback".to_string()],
            }),
            Rc::new(PlanCycleRequiredSourceRule {
                message: "plan cycle needs an entry".to_string(),
            }),
        ],
    }
}
