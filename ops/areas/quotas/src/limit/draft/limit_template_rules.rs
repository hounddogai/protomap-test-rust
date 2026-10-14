use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit template must pass. Check returns its problems, or none.
pub trait LimitTemplateRule: std::fmt::Debug {
    fn check(&self, item: &LimitTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTemplateCodeLengthRule {
    pub max_length: i64,
}

impl LimitTemplateRule for LimitTemplateCodeLengthRule {
    fn check(&self, item: &LimitTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTemplateRequiredTitleRule {
    pub message: String,
}

impl LimitTemplateRule for LimitTemplateRequiredTitleRule {
    fn check(&self, item: &LimitTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitTemplateCodePrefixRule {
    pub prefix: String,
}

impl LimitTemplateRule for LimitTemplateCodePrefixRule {
    fn check(&self, item: &LimitTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit template.
#[derive(Debug, Clone)]
pub struct LimitTemplateValidator {
    pub rules: Vec<Rc<dyn LimitTemplateRule>>,
}

impl LimitTemplateValidator {
    pub fn validate(&self, item: &LimitTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_template_validator() -> LimitTemplateValidator {
    LimitTemplateValidator {
        rules: vec![
            Rc::new(LimitTemplateCodeLengthRule { max_length: 32 }) as Rc<dyn LimitTemplateRule>,
            Rc::new(LimitTemplateRequiredTitleRule {
                message: "limit template needs this field".to_string(),
            }),
            Rc::new(LimitTemplateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
