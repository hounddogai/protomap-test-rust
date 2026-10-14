use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe milestone must pass. Check returns its problems, or none.
pub trait ProbeMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ProbeMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMilestoneRequiredNamespaceRule {
    pub message: String,
}

impl ProbeMilestoneRule for ProbeMilestoneRequiredNamespaceRule {
    fn check(&self, item: &ProbeMilestone) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMilestoneMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeMilestoneRule for ProbeMilestoneMinimumRangeRule {
    fn check(&self, item: &ProbeMilestone) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMilestoneNameLengthRule {
    pub max_length: i64,
}

impl ProbeMilestoneRule for ProbeMilestoneNameLengthRule {
    fn check(&self, item: &ProbeMilestone) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe milestone.
#[derive(Debug, Clone)]
pub struct ProbeMilestoneValidator {
    pub rules: Vec<Rc<dyn ProbeMilestoneRule>>,
}

impl ProbeMilestoneValidator {
    pub fn validate(&self, item: &ProbeMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_milestone_validator() -> ProbeMilestoneValidator {
    ProbeMilestoneValidator {
        rules: vec![
            Rc::new(ProbeMilestoneRequiredNamespaceRule {
                message: "probe milestone needs an entry".to_string(),
            }) as Rc<dyn ProbeMilestoneRule>,
            Rc::new(ProbeMilestoneMinimumRangeRule { min: 6.7, max: 4943.5 }),
            Rc::new(ProbeMilestoneNameLengthRule { max_length: 86400 }),
        ],
    }
}
