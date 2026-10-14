use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled evidence template must pass. Check returns its problems, or none.
pub trait ScheduledEvidenceTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledEvidenceTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTemplateCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledEvidenceTemplateRule for ScheduledEvidenceTemplateCodeLengthRule {
    fn check(&self, item: &ScheduledEvidenceTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTemplateRequiredNameRule {
    pub message: String,
}

impl ScheduledEvidenceTemplateRule for ScheduledEvidenceTemplateRequiredNameRule {
    fn check(&self, item: &ScheduledEvidenceTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTemplateRequiredCodeRule {
    pub message: String,
}

impl ScheduledEvidenceTemplateRule for ScheduledEvidenceTemplateRequiredCodeRule {
    fn check(&self, item: &ScheduledEvidenceTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTemplateNameLengthRule {
    pub max_length: i64,
}

impl ScheduledEvidenceTemplateRule for ScheduledEvidenceTemplateNameLengthRule {
    fn check(&self, item: &ScheduledEvidenceTemplate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTemplateCipherLengthRule {
    pub max_length: i64,
}

impl ScheduledEvidenceTemplateRule for ScheduledEvidenceTemplateCipherLengthRule {
    fn check(&self, item: &ScheduledEvidenceTemplate) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled evidence template.
#[derive(Debug, Clone)]
pub struct ScheduledEvidenceTemplateValidator {
    pub rules: Vec<Rc<dyn ScheduledEvidenceTemplateRule>>,
}

impl ScheduledEvidenceTemplateValidator {
    pub fn validate(&self, item: &ScheduledEvidenceTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledEvidenceTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledEvidenceTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_evidence_template_validator() -> ScheduledEvidenceTemplateValidator {
    ScheduledEvidenceTemplateValidator {
        rules: vec![
            Rc::new(ScheduledEvidenceTemplateCodeLengthRule { max_length: 86400 })
                as Rc<dyn ScheduledEvidenceTemplateRule>,
            Rc::new(ScheduledEvidenceTemplateRequiredNameRule {
                message: "scheduled evidence template needs this field".to_string(),
            }),
            Rc::new(ScheduledEvidenceTemplateRequiredCodeRule {
                message: "scheduled evidence template needs this field".to_string(),
            }),
            Rc::new(ScheduledEvidenceTemplateNameLengthRule { max_length: 16 }),
            Rc::new(ScheduledEvidenceTemplateCipherLengthRule { max_length: 3600 }),
        ],
    }
}
