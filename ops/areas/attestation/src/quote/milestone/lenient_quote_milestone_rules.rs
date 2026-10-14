use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient quote milestone must pass. Check returns its problems, or none.
pub trait LenientQuoteMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LenientQuoteMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteMilestoneCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientQuoteMilestoneRule for LenientQuoteMilestoneCeilingRangeRule {
    fn check(&self, item: &LenientQuoteMilestone) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteMilestoneRequiredNameRule {
    pub message: String,
}

impl LenientQuoteMilestoneRule for LenientQuoteMilestoneRequiredNameRule {
    fn check(&self, item: &LenientQuoteMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteMilestonePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientQuoteMilestoneRule for LenientQuoteMilestonePriorityRangeRule {
    fn check(&self, item: &LenientQuoteMilestone) -> Vec<String> {
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

/// Runs every rule of a lenient quote milestone.
#[derive(Debug, Clone)]
pub struct LenientQuoteMilestoneValidator {
    pub rules: Vec<Rc<dyn LenientQuoteMilestoneRule>>,
}

impl LenientQuoteMilestoneValidator {
    pub fn validate(&self, item: &LenientQuoteMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientQuoteMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientQuoteMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_quote_milestone_validator() -> LenientQuoteMilestoneValidator {
    LenientQuoteMilestoneValidator {
        rules: vec![
            Rc::new(LenientQuoteMilestoneCeilingRangeRule { min: 3.4, max: 4479.3 })
                as Rc<dyn LenientQuoteMilestoneRule>,
            Rc::new(LenientQuoteMilestoneRequiredNameRule {
                message: "lenient quote milestone needs an entry".to_string(),
            }),
            Rc::new(LenientQuoteMilestonePriorityRangeRule { min: 32, max: 64 }),
        ],
    }
}
