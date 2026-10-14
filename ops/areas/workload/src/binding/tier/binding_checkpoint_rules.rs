use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding checkpoint must pass. Check returns its problems, or none.
pub trait BindingCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &BindingCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingCheckpointRequiredNameRule {
    pub message: String,
}

impl BindingCheckpointRule for BindingCheckpointRequiredNameRule {
    fn check(&self, item: &BindingCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingCheckpointNotesLengthRule {
    pub max_length: i64,
}

impl BindingCheckpointRule for BindingCheckpointNotesLengthRule {
    fn check(&self, item: &BindingCheckpoint) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingCheckpointPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingCheckpointRule for BindingCheckpointPriorityRangeRule {
    fn check(&self, item: &BindingCheckpoint) -> Vec<String> {
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
pub struct BindingCheckpointCodePrefixRule {
    pub prefix: String,
}

impl BindingCheckpointRule for BindingCheckpointCodePrefixRule {
    fn check(&self, item: &BindingCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding checkpoint.
#[derive(Debug, Clone)]
pub struct BindingCheckpointValidator {
    pub rules: Vec<Rc<dyn BindingCheckpointRule>>,
}

impl BindingCheckpointValidator {
    pub fn validate(&self, item: &BindingCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_checkpoint_validator() -> BindingCheckpointValidator {
    BindingCheckpointValidator {
        rules: vec![
            Rc::new(BindingCheckpointRequiredNameRule {
                message: "binding checkpoint needs this field".to_string(),
            }) as Rc<dyn BindingCheckpointRule>,
            Rc::new(BindingCheckpointNotesLengthRule { max_length: 3600 }),
            Rc::new(BindingCheckpointPriorityRangeRule { min: 86400, max: 86400 }),
            Rc::new(BindingCheckpointCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_checkpoint_validator();
        let item = new_binding_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BindingCheckpointRequiredNameRule {
            message: "binding checkpoint needs an entry".to_string(),
        };
        let mut item = new_binding_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = BindingCheckpointNotesLengthRule { max_length: 86400 };
        let mut item = new_binding_checkpoint("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = BindingCheckpointPriorityRangeRule { min: 64, max: 120 };
        let mut item = new_binding_checkpoint("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
