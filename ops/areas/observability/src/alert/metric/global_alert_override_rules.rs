use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global alert override must pass. Check returns its problems, or none.
pub trait GlobalAlertOverrideRule: std::fmt::Debug {
    fn check(&self, item: &GlobalAlertOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertOverrideRequiredNamespaceRule {
    pub message: String,
}

impl GlobalAlertOverrideRule for GlobalAlertOverrideRequiredNamespaceRule {
    fn check(&self, item: &GlobalAlertOverride) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertOverrideTitleLengthRule {
    pub max_length: i64,
}

impl GlobalAlertOverrideRule for GlobalAlertOverrideTitleLengthRule {
    fn check(&self, item: &GlobalAlertOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertOverrideCodePrefixRule {
    pub prefix: String,
}

impl GlobalAlertOverrideRule for GlobalAlertOverrideCodePrefixRule {
    fn check(&self, item: &GlobalAlertOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertOverrideCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalAlertOverrideRule for GlobalAlertOverrideCapacityRangeRule {
    fn check(&self, item: &GlobalAlertOverride) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertOverrideRegionLengthRule {
    pub max_length: i64,
}

impl GlobalAlertOverrideRule for GlobalAlertOverrideRegionLengthRule {
    fn check(&self, item: &GlobalAlertOverride) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global alert override.
#[derive(Debug, Clone)]
pub struct GlobalAlertOverrideValidator {
    pub rules: Vec<Rc<dyn GlobalAlertOverrideRule>>,
}

impl GlobalAlertOverrideValidator {
    pub fn validate(&self, item: &GlobalAlertOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalAlertOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalAlertOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_alert_override_validator() -> GlobalAlertOverrideValidator {
    GlobalAlertOverrideValidator {
        rules: vec![
            Rc::new(GlobalAlertOverrideRequiredNamespaceRule {
                message: "global alert override needs this field".to_string(),
            }) as Rc<dyn GlobalAlertOverrideRule>,
            Rc::new(GlobalAlertOverrideTitleLengthRule { max_length: 3600 }),
            Rc::new(GlobalAlertOverrideCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GlobalAlertOverrideCapacityRangeRule { min: 255, max: 3600 }),
            Rc::new(GlobalAlertOverrideRegionLengthRule { max_length: 16 }),
        ],
    }
}
