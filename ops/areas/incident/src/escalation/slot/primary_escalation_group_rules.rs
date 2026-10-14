use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary escalation group must pass. Check returns its problems, or none.
pub trait PrimaryEscalationGroupRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEscalationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationGroupPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEscalationGroupRule for PrimaryEscalationGroupPortRangeRule {
    fn check(&self, item: &PrimaryEscalationGroup) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationGroupScopeLengthRule {
    pub max_length: i64,
}

impl PrimaryEscalationGroupRule for PrimaryEscalationGroupScopeLengthRule {
    fn check(&self, item: &PrimaryEscalationGroup) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationGroupRequiredNameRule {
    pub message: String,
}

impl PrimaryEscalationGroupRule for PrimaryEscalationGroupRequiredNameRule {
    fn check(&self, item: &PrimaryEscalationGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary escalation group.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationGroupValidator {
    pub rules: Vec<Rc<dyn PrimaryEscalationGroupRule>>,
}

impl PrimaryEscalationGroupValidator {
    pub fn validate(&self, item: &PrimaryEscalationGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryEscalationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEscalationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_escalation_group_validator() -> PrimaryEscalationGroupValidator {
    PrimaryEscalationGroupValidator {
        rules: vec![
            Rc::new(PrimaryEscalationGroupPortRangeRule { min: 120, max: 86400 }) as Rc<dyn PrimaryEscalationGroupRule>,
            Rc::new(PrimaryEscalationGroupScopeLengthRule { max_length: 86400 }),
            Rc::new(PrimaryEscalationGroupRequiredNameRule {
                message: "primary escalation group needs this field".to_string(),
            }),
        ],
    }
}
