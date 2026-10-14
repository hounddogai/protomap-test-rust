use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction template must pass. Check returns its problems, or none.
pub trait RedactionTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RedactionTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTemplateCipherLengthRule {
    pub max_length: i64,
}

impl RedactionTemplateRule for RedactionTemplateCipherLengthRule {
    fn check(&self, item: &RedactionTemplate) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTemplateCodeLengthRule {
    pub max_length: i64,
}

impl RedactionTemplateRule for RedactionTemplateCodeLengthRule {
    fn check(&self, item: &RedactionTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTemplateCodePrefixRule {
    pub prefix: String,
}

impl RedactionTemplateRule for RedactionTemplateCodePrefixRule {
    fn check(&self, item: &RedactionTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTemplateLocaleLengthRule {
    pub max_length: i64,
}

impl RedactionTemplateRule for RedactionTemplateLocaleLengthRule {
    fn check(&self, item: &RedactionTemplate) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTemplateLabelLengthRule {
    pub max_length: i64,
}

impl RedactionTemplateRule for RedactionTemplateLabelLengthRule {
    fn check(&self, item: &RedactionTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction template.
#[derive(Debug, Clone)]
pub struct RedactionTemplateValidator {
    pub rules: Vec<Rc<dyn RedactionTemplateRule>>,
}

impl RedactionTemplateValidator {
    pub fn validate(&self, item: &RedactionTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_template_validator() -> RedactionTemplateValidator {
    RedactionTemplateValidator {
        rules: vec![
            Rc::new(RedactionTemplateCipherLengthRule { max_length: 16 }) as Rc<dyn RedactionTemplateRule>,
            Rc::new(RedactionTemplateCodeLengthRule { max_length: 16 }),
            Rc::new(RedactionTemplateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RedactionTemplateLocaleLengthRule { max_length: 64 }),
            Rc::new(RedactionTemplateLabelLengthRule { max_length: 86400 }),
        ],
    }
}
