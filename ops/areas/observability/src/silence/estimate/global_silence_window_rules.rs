use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global silence window must pass. Check returns its problems, or none.
pub trait GlobalSilenceWindowRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSilenceWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceWindowAudienceLengthRule {
    pub max_length: i64,
}

impl GlobalSilenceWindowRule for GlobalSilenceWindowAudienceLengthRule {
    fn check(&self, item: &GlobalSilenceWindow) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceWindowCodeLengthRule {
    pub max_length: i64,
}

impl GlobalSilenceWindowRule for GlobalSilenceWindowCodeLengthRule {
    fn check(&self, item: &GlobalSilenceWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceWindowActiveGuardRule {
    pub limit: i64,
}

impl GlobalSilenceWindowRule for GlobalSilenceWindowActiveGuardRule {
    fn check(&self, item: &GlobalSilenceWindow) -> Vec<String> {
        if item.active && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while active",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceWindowLabelLengthRule {
    pub max_length: i64,
}

impl GlobalSilenceWindowRule for GlobalSilenceWindowLabelLengthRule {
    fn check(&self, item: &GlobalSilenceWindow) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global silence window.
#[derive(Debug, Clone)]
pub struct GlobalSilenceWindowValidator {
    pub rules: Vec<Rc<dyn GlobalSilenceWindowRule>>,
}

impl GlobalSilenceWindowValidator {
    pub fn validate(&self, item: &GlobalSilenceWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalSilenceWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSilenceWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_silence_window_validator() -> GlobalSilenceWindowValidator {
    GlobalSilenceWindowValidator {
        rules: vec![
            Rc::new(GlobalSilenceWindowAudienceLengthRule { max_length: 1000 }) as Rc<dyn GlobalSilenceWindowRule>,
            Rc::new(GlobalSilenceWindowCodeLengthRule { max_length: 1000 }),
            Rc::new(GlobalSilenceWindowActiveGuardRule { limit: 120 }),
            Rc::new(GlobalSilenceWindowLabelLengthRule { max_length: 32 }),
        ],
    }
}
