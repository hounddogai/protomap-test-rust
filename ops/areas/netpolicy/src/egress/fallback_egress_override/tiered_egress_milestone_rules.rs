use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered egress milestone must pass. Check returns its problems, or none.
pub trait TieredEgressMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &TieredEgressMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressMilestoneHostLengthRule {
    pub max_length: i64,
}

impl TieredEgressMilestoneRule for TieredEgressMilestoneHostLengthRule {
    fn check(&self, item: &TieredEgressMilestone) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressMilestoneNotesLengthRule {
    pub max_length: i64,
}

impl TieredEgressMilestoneRule for TieredEgressMilestoneNotesLengthRule {
    fn check(&self, item: &TieredEgressMilestone) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressMilestoneAllowedHostRule {
    pub allowed: Vec<String>,
}

impl TieredEgressMilestoneRule for TieredEgressMilestoneAllowedHostRule {
    fn check(&self, item: &TieredEgressMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is above maximum", item.host)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered egress milestone.
#[derive(Debug, Clone)]
pub struct TieredEgressMilestoneValidator {
    pub rules: Vec<Rc<dyn TieredEgressMilestoneRule>>,
}

impl TieredEgressMilestoneValidator {
    pub fn validate(&self, item: &TieredEgressMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredEgressMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredEgressMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_egress_milestone_validator() -> TieredEgressMilestoneValidator {
    TieredEgressMilestoneValidator {
        rules: vec![
            Rc::new(TieredEgressMilestoneHostLengthRule { max_length: 80 }) as Rc<dyn TieredEgressMilestoneRule>,
            Rc::new(TieredEgressMilestoneNotesLengthRule { max_length: 32 }),
            Rc::new(TieredEgressMilestoneAllowedHostRule {
                allowed: vec!["manual".to_string(), "global".to_string(), "capped".to_string()],
            }),
        ],
    }
}
