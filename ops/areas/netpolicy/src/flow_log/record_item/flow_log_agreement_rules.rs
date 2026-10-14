use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log agreement must pass. Check returns its problems, or none.
pub trait FlowLogAgreementRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAgreementCodePrefixRule {
    pub prefix: String,
}

impl FlowLogAgreementRule for FlowLogAgreementCodePrefixRule {
    fn check(&self, item: &FlowLogAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAgreementLabelLengthRule {
    pub max_length: i64,
}

impl FlowLogAgreementRule for FlowLogAgreementLabelLengthRule {
    fn check(&self, item: &FlowLogAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAgreementCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogAgreementRule for FlowLogAgreementCeilingRangeRule {
    fn check(&self, item: &FlowLogAgreement) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAgreementCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogAgreementRule for FlowLogAgreementCodeLengthRule {
    fn check(&self, item: &FlowLogAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log agreement.
#[derive(Debug, Clone)]
pub struct FlowLogAgreementValidator {
    pub rules: Vec<Rc<dyn FlowLogAgreementRule>>,
}

impl FlowLogAgreementValidator {
    pub fn validate(&self, item: &FlowLogAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_agreement_validator() -> FlowLogAgreementValidator {
    FlowLogAgreementValidator {
        rules: vec![
            Rc::new(FlowLogAgreementCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn FlowLogAgreementRule>,
            Rc::new(FlowLogAgreementLabelLengthRule { max_length: 3600 }),
            Rc::new(FlowLogAgreementCeilingRangeRule { min: 4.7, max: 3442.9 }),
            Rc::new(FlowLogAgreementCodeLengthRule { max_length: 16 }),
        ],
    }
}
