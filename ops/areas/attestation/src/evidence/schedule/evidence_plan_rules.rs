use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence plan must pass. Check returns its problems, or none.
pub trait EvidencePlanRule: std::fmt::Debug {
    fn check(&self, item: &EvidencePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePlanAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl EvidencePlanRule for EvidencePlanAllowedTitleRule {
    fn check(&self, item: &EvidencePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePlanTitleLengthRule {
    pub max_length: i64,
}

impl EvidencePlanRule for EvidencePlanTitleLengthRule {
    fn check(&self, item: &EvidencePlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePlanPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidencePlanRule for EvidencePlanPortRangeRule {
    fn check(&self, item: &EvidencePlan) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePlanCodePrefixRule {
    pub prefix: String,
}

impl EvidencePlanRule for EvidencePlanCodePrefixRule {
    fn check(&self, item: &EvidencePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePlanRequiredCodeRule {
    pub message: String,
}

impl EvidencePlanRule for EvidencePlanRequiredCodeRule {
    fn check(&self, item: &EvidencePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePlanCodeLengthRule {
    pub max_length: i64,
}

impl EvidencePlanRule for EvidencePlanCodeLengthRule {
    fn check(&self, item: &EvidencePlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence plan.
#[derive(Debug, Clone)]
pub struct EvidencePlanValidator {
    pub rules: Vec<Rc<dyn EvidencePlanRule>>,
}

impl EvidencePlanValidator {
    pub fn validate(&self, item: &EvidencePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidencePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidencePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_plan_validator() -> EvidencePlanValidator {
    EvidencePlanValidator {
        rules: vec![
            Rc::new(EvidencePlanAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn EvidencePlanRule>,
            Rc::new(EvidencePlanTitleLengthRule { max_length: 32 }),
            Rc::new(EvidencePlanPortRangeRule { min: 80, max: 3600 }),
            Rc::new(EvidencePlanCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(EvidencePlanRequiredCodeRule {
                message: "evidence plan needs this field".to_string(),
            }),
            Rc::new(EvidencePlanCodeLengthRule { max_length: 80 }),
        ],
    }
}
