use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary budget burn cycle must pass. Check returns its problems, or none.
pub trait PrimaryBudgetBurnCycleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryBudgetBurnCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnCycleRequiredChannelRule {
    pub message: String,
}

impl PrimaryBudgetBurnCycleRule for PrimaryBudgetBurnCycleRequiredChannelRule {
    fn check(&self, item: &PrimaryBudgetBurnCycle) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnCycleCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryBudgetBurnCycleRule for PrimaryBudgetBurnCycleCodeLengthRule {
    fn check(&self, item: &PrimaryBudgetBurnCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnCycleHostLengthRule {
    pub max_length: i64,
}

impl PrimaryBudgetBurnCycleRule for PrimaryBudgetBurnCycleHostLengthRule {
    fn check(&self, item: &PrimaryBudgetBurnCycle) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary budget burn cycle.
#[derive(Debug, Clone)]
pub struct PrimaryBudgetBurnCycleValidator {
    pub rules: Vec<Rc<dyn PrimaryBudgetBurnCycleRule>>,
}

impl PrimaryBudgetBurnCycleValidator {
    pub fn validate(&self, item: &PrimaryBudgetBurnCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryBudgetBurnCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryBudgetBurnCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_budget_burn_cycle_validator() -> PrimaryBudgetBurnCycleValidator {
    PrimaryBudgetBurnCycleValidator {
        rules: vec![
            Rc::new(PrimaryBudgetBurnCycleRequiredChannelRule {
                message: "primary budget burn cycle needs this field".to_string(),
            }) as Rc<dyn PrimaryBudgetBurnCycleRule>,
            Rc::new(PrimaryBudgetBurnCycleCodeLengthRule { max_length: 64 }),
            Rc::new(PrimaryBudgetBurnCycleHostLengthRule { max_length: 16 }),
        ],
    }
}
