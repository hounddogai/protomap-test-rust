use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log mapping must pass. Check returns its problems, or none.
pub trait FlowLogMappingRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMappingLabelLengthRule {
    pub max_length: i64,
}

impl FlowLogMappingRule for FlowLogMappingLabelLengthRule {
    fn check(&self, item: &FlowLogMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMappingCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogMappingRule for FlowLogMappingCodeLengthRule {
    fn check(&self, item: &FlowLogMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMappingRequiredUnitRule {
    pub message: String,
}

impl FlowLogMappingRule for FlowLogMappingRequiredUnitRule {
    fn check(&self, item: &FlowLogMapping) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMappingCodePrefixRule {
    pub prefix: String,
}

impl FlowLogMappingRule for FlowLogMappingCodePrefixRule {
    fn check(&self, item: &FlowLogMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log mapping.
#[derive(Debug, Clone)]
pub struct FlowLogMappingValidator {
    pub rules: Vec<Rc<dyn FlowLogMappingRule>>,
}

impl FlowLogMappingValidator {
    pub fn validate(&self, item: &FlowLogMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_mapping_validator() -> FlowLogMappingValidator {
    FlowLogMappingValidator {
        rules: vec![
            Rc::new(FlowLogMappingLabelLengthRule { max_length: 80 }) as Rc<dyn FlowLogMappingRule>,
            Rc::new(FlowLogMappingCodeLengthRule { max_length: 1000 }),
            Rc::new(FlowLogMappingRequiredUnitRule {
                message: "flow log mapping needs a value".to_string(),
            }),
            Rc::new(FlowLogMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
