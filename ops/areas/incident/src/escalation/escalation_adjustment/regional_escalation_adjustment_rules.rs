use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional escalation adjustment must pass. Check returns its problems, or none.
pub trait RegionalEscalationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalEscalationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationAdjustmentBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalEscalationAdjustmentRule for RegionalEscalationAdjustmentBudgetRangeRule {
    fn check(&self, item: &RegionalEscalationAdjustment) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl RegionalEscalationAdjustmentRule for RegionalEscalationAdjustmentTitleLengthRule {
    fn check(&self, item: &RegionalEscalationAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl RegionalEscalationAdjustmentRule for RegionalEscalationAdjustmentCodeLengthRule {
    fn check(&self, item: &RegionalEscalationAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional escalation adjustment.
#[derive(Debug, Clone)]
pub struct RegionalEscalationAdjustmentValidator {
    pub rules: Vec<Rc<dyn RegionalEscalationAdjustmentRule>>,
}

impl RegionalEscalationAdjustmentValidator {
    pub fn validate(&self, item: &RegionalEscalationAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalEscalationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalEscalationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_escalation_adjustment_validator() -> RegionalEscalationAdjustmentValidator {
    RegionalEscalationAdjustmentValidator {
        rules: vec![
            Rc::new(RegionalEscalationAdjustmentBudgetRangeRule { min: 2.3, max: 3755.4 })
                as Rc<dyn RegionalEscalationAdjustmentRule>,
            Rc::new(RegionalEscalationAdjustmentTitleLengthRule { max_length: 64 }),
            Rc::new(RegionalEscalationAdjustmentCodeLengthRule { max_length: 32 }),
        ],
    }
}
