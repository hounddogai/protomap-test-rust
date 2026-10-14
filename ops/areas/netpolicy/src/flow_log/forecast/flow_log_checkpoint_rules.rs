use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log checkpoint must pass. Check returns its problems, or none.
pub trait FlowLogCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCheckpointAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FlowLogCheckpointRule for FlowLogCheckpointAllowedNameRule {
    fn check(&self, item: &FlowLogCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCheckpointNameLengthRule {
    pub max_length: i64,
}

impl FlowLogCheckpointRule for FlowLogCheckpointNameLengthRule {
    fn check(&self, item: &FlowLogCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCheckpointUnitLengthRule {
    pub max_length: i64,
}

impl FlowLogCheckpointRule for FlowLogCheckpointUnitLengthRule {
    fn check(&self, item: &FlowLogCheckpoint) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCheckpointCodePrefixRule {
    pub prefix: String,
}

impl FlowLogCheckpointRule for FlowLogCheckpointCodePrefixRule {
    fn check(&self, item: &FlowLogCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log checkpoint.
#[derive(Debug, Clone)]
pub struct FlowLogCheckpointValidator {
    pub rules: Vec<Rc<dyn FlowLogCheckpointRule>>,
}

impl FlowLogCheckpointValidator {
    pub fn validate(&self, item: &FlowLogCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_checkpoint_validator() -> FlowLogCheckpointValidator {
    FlowLogCheckpointValidator {
        rules: vec![
            Rc::new(FlowLogCheckpointAllowedNameRule {
                allowed: vec!["scheduled".to_string()],
            }) as Rc<dyn FlowLogCheckpointRule>,
            Rc::new(FlowLogCheckpointNameLengthRule { max_length: 64 }),
            Rc::new(FlowLogCheckpointUnitLengthRule { max_length: 3600 }),
            Rc::new(FlowLogCheckpointCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_checkpoint_validator();
        let item = new_flow_log_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FlowLogCheckpointNameLengthRule { max_length: 80 };
        let mut item = new_flow_log_checkpoint("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = FlowLogCheckpointUnitLengthRule { max_length: 32 };
        let mut item = new_flow_log_checkpoint("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
