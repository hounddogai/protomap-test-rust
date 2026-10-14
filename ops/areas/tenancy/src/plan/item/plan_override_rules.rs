use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan override must pass. Check returns its problems, or none.
pub trait PlanOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PlanOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanOverrideRequiredNotesRule {
    pub message: String,
}

impl PlanOverrideRule for PlanOverrideRequiredNotesRule {
    fn check(&self, item: &PlanOverride) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PlanOverrideRule for PlanOverrideAllowedCodeRule {
    fn check(&self, item: &PlanOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanOverridePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanOverrideRule for PlanOverridePriorityRangeRule {
    fn check(&self, item: &PlanOverride) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanOverrideBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanOverrideRule for PlanOverrideBudgetRangeRule {
    fn check(&self, item: &PlanOverride) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanOverrideBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanOverrideRule for PlanOverrideBatchSizeRangeRule {
    fn check(&self, item: &PlanOverride) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanOverrideRequiredChecksumRule {
    pub message: String,
}

impl PlanOverrideRule for PlanOverrideRequiredChecksumRule {
    fn check(&self, item: &PlanOverride) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan override.
#[derive(Debug, Clone)]
pub struct PlanOverrideValidator {
    pub rules: Vec<Rc<dyn PlanOverrideRule>>,
}

impl PlanOverrideValidator {
    pub fn validate(&self, item: &PlanOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_override_validator() -> PlanOverrideValidator {
    PlanOverrideValidator {
        rules: vec![
            Rc::new(PlanOverrideRequiredNotesRule {
                message: "plan override needs a value".to_string(),
            }) as Rc<dyn PlanOverrideRule>,
            Rc::new(PlanOverrideAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "legacy".to_string()],
            }),
            Rc::new(PlanOverridePriorityRangeRule { min: 80, max: 1000 }),
            Rc::new(PlanOverrideBudgetRangeRule { min: 9.1, max: 915.5 }),
            Rc::new(PlanOverrideBatchSizeRangeRule { min: 32, max: 86400 }),
            Rc::new(PlanOverrideRequiredChecksumRule {
                message: "plan override needs an entry".to_string(),
            }),
        ],
    }
}
