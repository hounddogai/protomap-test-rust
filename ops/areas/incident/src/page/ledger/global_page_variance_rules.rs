use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global page variance must pass. Check returns its problems, or none.
pub trait GlobalPageVarianceRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPageVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageVarianceCodePrefixRule {
    pub prefix: String,
}

impl GlobalPageVarianceRule for GlobalPageVarianceCodePrefixRule {
    fn check(&self, item: &GlobalPageVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageVarianceRequiredLabelRule {
    pub message: String,
}

impl GlobalPageVarianceRule for GlobalPageVarianceRequiredLabelRule {
    fn check(&self, item: &GlobalPageVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageVarianceCodeLengthRule {
    pub max_length: i64,
}

impl GlobalPageVarianceRule for GlobalPageVarianceCodeLengthRule {
    fn check(&self, item: &GlobalPageVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageVarianceRequiredNameRule {
    pub message: String,
}

impl GlobalPageVarianceRule for GlobalPageVarianceRequiredNameRule {
    fn check(&self, item: &GlobalPageVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageVarianceLabelLengthRule {
    pub max_length: i64,
}

impl GlobalPageVarianceRule for GlobalPageVarianceLabelLengthRule {
    fn check(&self, item: &GlobalPageVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global page variance.
#[derive(Debug, Clone)]
pub struct GlobalPageVarianceValidator {
    pub rules: Vec<Rc<dyn GlobalPageVarianceRule>>,
}

impl GlobalPageVarianceValidator {
    pub fn validate(&self, item: &GlobalPageVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPageVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPageVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_page_variance_validator() -> GlobalPageVarianceValidator {
    GlobalPageVarianceValidator {
        rules: vec![
            Rc::new(GlobalPageVarianceCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn GlobalPageVarianceRule>,
            Rc::new(GlobalPageVarianceRequiredLabelRule {
                message: "global page variance needs a value".to_string(),
            }),
            Rc::new(GlobalPageVarianceCodeLengthRule { max_length: 120 }),
            Rc::new(GlobalPageVarianceRequiredNameRule {
                message: "global page variance needs an entry".to_string(),
            }),
            Rc::new(GlobalPageVarianceLabelLengthRule { max_length: 120 }),
        ],
    }
}
