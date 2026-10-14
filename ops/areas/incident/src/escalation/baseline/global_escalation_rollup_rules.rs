use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global escalation rollup must pass. Check returns its problems, or none.
pub trait GlobalEscalationRollupRule: std::fmt::Debug {
    fn check(&self, item: &GlobalEscalationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationRollupAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl GlobalEscalationRollupRule for GlobalEscalationRollupAllowedTitleRule {
    fn check(&self, item: &GlobalEscalationRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationRollupCodePrefixRule {
    pub prefix: String,
}

impl GlobalEscalationRollupRule for GlobalEscalationRollupCodePrefixRule {
    fn check(&self, item: &GlobalEscalationRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationRollupRequiredTitleRule {
    pub message: String,
}

impl GlobalEscalationRollupRule for GlobalEscalationRollupRequiredTitleRule {
    fn check(&self, item: &GlobalEscalationRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global escalation rollup.
#[derive(Debug, Clone)]
pub struct GlobalEscalationRollupValidator {
    pub rules: Vec<Rc<dyn GlobalEscalationRollupRule>>,
}

impl GlobalEscalationRollupValidator {
    pub fn validate(&self, item: &GlobalEscalationRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalEscalationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalEscalationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_escalation_rollup_validator() -> GlobalEscalationRollupValidator {
    GlobalEscalationRollupValidator {
        rules: vec![
            Rc::new(GlobalEscalationRollupAllowedTitleRule { allowed: Vec::new() })
                as Rc<dyn GlobalEscalationRollupRule>,
            Rc::new(GlobalEscalationRollupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(GlobalEscalationRollupRequiredTitleRule {
                message: "global escalation rollup needs an entry".to_string(),
            }),
        ],
    }
}
