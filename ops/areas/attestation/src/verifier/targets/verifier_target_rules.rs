use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier target must pass. Check returns its problems, or none.
pub trait VerifierTargetRule: std::fmt::Debug {
    fn check(&self, item: &VerifierTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTargetDescriptionLengthRule {
    pub max_length: i64,
}

impl VerifierTargetRule for VerifierTargetDescriptionLengthRule {
    fn check(&self, item: &VerifierTarget) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTargetRequiredDescriptionRule {
    pub message: String,
}

impl VerifierTargetRule for VerifierTargetRequiredDescriptionRule {
    fn check(&self, item: &VerifierTarget) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTargetRequiredCodeRule {
    pub message: String,
}

impl VerifierTargetRule for VerifierTargetRequiredCodeRule {
    fn check(&self, item: &VerifierTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTargetNameLengthRule {
    pub max_length: i64,
}

impl VerifierTargetRule for VerifierTargetNameLengthRule {
    fn check(&self, item: &VerifierTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTargetCodePrefixRule {
    pub prefix: String,
}

impl VerifierTargetRule for VerifierTargetCodePrefixRule {
    fn check(&self, item: &VerifierTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier target.
#[derive(Debug, Clone)]
pub struct VerifierTargetValidator {
    pub rules: Vec<Rc<dyn VerifierTargetRule>>,
}

impl VerifierTargetValidator {
    pub fn validate(&self, item: &VerifierTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_target_validator() -> VerifierTargetValidator {
    VerifierTargetValidator {
        rules: vec![
            Rc::new(VerifierTargetDescriptionLengthRule { max_length: 120 }) as Rc<dyn VerifierTargetRule>,
            Rc::new(VerifierTargetRequiredDescriptionRule {
                message: "verifier target needs this field".to_string(),
            }),
            Rc::new(VerifierTargetRequiredCodeRule {
                message: "verifier target needs this field".to_string(),
            }),
            Rc::new(VerifierTargetNameLengthRule { max_length: 255 }),
            Rc::new(VerifierTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
