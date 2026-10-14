use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped redaction template must pass. Check returns its problems, or none.
pub trait CappedRedactionTemplateRule: std::fmt::Debug {
    fn check(&self, item: &CappedRedactionTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionTemplateCipherLengthRule {
    pub max_length: i64,
}

impl CappedRedactionTemplateRule for CappedRedactionTemplateCipherLengthRule {
    fn check(&self, item: &CappedRedactionTemplate) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionTemplateRequiredCodeRule {
    pub message: String,
}

impl CappedRedactionTemplateRule for CappedRedactionTemplateRequiredCodeRule {
    fn check(&self, item: &CappedRedactionTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionTemplateRequiredNotesRule {
    pub message: String,
}

impl CappedRedactionTemplateRule for CappedRedactionTemplateRequiredNotesRule {
    fn check(&self, item: &CappedRedactionTemplate) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionTemplateCodePrefixRule {
    pub prefix: String,
}

impl CappedRedactionTemplateRule for CappedRedactionTemplateCodePrefixRule {
    fn check(&self, item: &CappedRedactionTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionTemplateNotesLengthRule {
    pub max_length: i64,
}

impl CappedRedactionTemplateRule for CappedRedactionTemplateNotesLengthRule {
    fn check(&self, item: &CappedRedactionTemplate) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionTemplateAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedRedactionTemplateRule for CappedRedactionTemplateAmountRangeRule {
    fn check(&self, item: &CappedRedactionTemplate) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped redaction template.
#[derive(Debug, Clone)]
pub struct CappedRedactionTemplateValidator {
    pub rules: Vec<Rc<dyn CappedRedactionTemplateRule>>,
}

impl CappedRedactionTemplateValidator {
    pub fn validate(&self, item: &CappedRedactionTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRedactionTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRedactionTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_redaction_template_validator() -> CappedRedactionTemplateValidator {
    CappedRedactionTemplateValidator {
        rules: vec![
            Rc::new(CappedRedactionTemplateCipherLengthRule { max_length: 32 }) as Rc<dyn CappedRedactionTemplateRule>,
            Rc::new(CappedRedactionTemplateRequiredCodeRule {
                message: "capped redaction template needs an entry".to_string(),
            }),
            Rc::new(CappedRedactionTemplateRequiredNotesRule {
                message: "capped redaction template needs a value".to_string(),
            }),
            Rc::new(CappedRedactionTemplateCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedRedactionTemplateNotesLengthRule { max_length: 255 }),
            Rc::new(CappedRedactionTemplateAmountRangeRule { min: 4.2, max: 4997.6 }),
        ],
    }
}
