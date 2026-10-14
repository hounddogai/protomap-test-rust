use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary redaction template must pass. Check returns its problems, or none.
pub trait PrimaryRedactionTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRedactionTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionTemplateCodePrefixRule {
    pub prefix: String,
}

impl PrimaryRedactionTemplateRule for PrimaryRedactionTemplateCodePrefixRule {
    fn check(&self, item: &PrimaryRedactionTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionTemplateRequiredNameRule {
    pub message: String,
}

impl PrimaryRedactionTemplateRule for PrimaryRedactionTemplateRequiredNameRule {
    fn check(&self, item: &PrimaryRedactionTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionTemplateCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRedactionTemplateRule for PrimaryRedactionTemplateCodeLengthRule {
    fn check(&self, item: &PrimaryRedactionTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionTemplateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRedactionTemplateRule for PrimaryRedactionTemplateBudgetRangeRule {
    fn check(&self, item: &PrimaryRedactionTemplate) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionTemplateIssuerLengthRule {
    pub max_length: i64,
}

impl PrimaryRedactionTemplateRule for PrimaryRedactionTemplateIssuerLengthRule {
    fn check(&self, item: &PrimaryRedactionTemplate) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionTemplateNameLengthRule {
    pub max_length: i64,
}

impl PrimaryRedactionTemplateRule for PrimaryRedactionTemplateNameLengthRule {
    fn check(&self, item: &PrimaryRedactionTemplate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary redaction template.
#[derive(Debug, Clone)]
pub struct PrimaryRedactionTemplateValidator {
    pub rules: Vec<Rc<dyn PrimaryRedactionTemplateRule>>,
}

impl PrimaryRedactionTemplateValidator {
    pub fn validate(&self, item: &PrimaryRedactionTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRedactionTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRedactionTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_redaction_template_validator() -> PrimaryRedactionTemplateValidator {
    PrimaryRedactionTemplateValidator {
        rules: vec![
            Rc::new(PrimaryRedactionTemplateCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn PrimaryRedactionTemplateRule>,
            Rc::new(PrimaryRedactionTemplateRequiredNameRule {
                message: "primary redaction template needs a value".to_string(),
            }),
            Rc::new(PrimaryRedactionTemplateCodeLengthRule { max_length: 3600 }),
            Rc::new(PrimaryRedactionTemplateBudgetRangeRule { min: 6.8, max: 4130.1 }),
            Rc::new(PrimaryRedactionTemplateIssuerLengthRule { max_length: 255 }),
            Rc::new(PrimaryRedactionTemplateNameLengthRule { max_length: 32 }),
        ],
    }
}
