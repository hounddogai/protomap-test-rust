use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual dns record template must pass. Check returns its problems, or none.
pub trait ManualDnsRecordTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ManualDnsRecordTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordTemplateTitleLengthRule {
    pub max_length: i64,
}

impl ManualDnsRecordTemplateRule for ManualDnsRecordTemplateTitleLengthRule {
    fn check(&self, item: &ManualDnsRecordTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordTemplateCodePrefixRule {
    pub prefix: String,
}

impl ManualDnsRecordTemplateRule for ManualDnsRecordTemplateCodePrefixRule {
    fn check(&self, item: &ManualDnsRecordTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordTemplateCodeLengthRule {
    pub max_length: i64,
}

impl ManualDnsRecordTemplateRule for ManualDnsRecordTemplateCodeLengthRule {
    fn check(&self, item: &ManualDnsRecordTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual dns record template.
#[derive(Debug, Clone)]
pub struct ManualDnsRecordTemplateValidator {
    pub rules: Vec<Rc<dyn ManualDnsRecordTemplateRule>>,
}

impl ManualDnsRecordTemplateValidator {
    pub fn validate(&self, item: &ManualDnsRecordTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualDnsRecordTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDnsRecordTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_dns_record_template_validator() -> ManualDnsRecordTemplateValidator {
    ManualDnsRecordTemplateValidator {
        rules: vec![
            Rc::new(ManualDnsRecordTemplateTitleLengthRule { max_length: 1000 }) as Rc<dyn ManualDnsRecordTemplateRule>,
            Rc::new(ManualDnsRecordTemplateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ManualDnsRecordTemplateCodeLengthRule { max_length: 86400 }),
        ],
    }
}
