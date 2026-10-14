use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page group must pass. Check returns its problems, or none.
pub trait PageGroupRule: std::fmt::Debug {
    fn check(&self, item: &PageGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageGroupCodeLengthRule {
    pub max_length: i64,
}

impl PageGroupRule for PageGroupCodeLengthRule {
    fn check(&self, item: &PageGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageGroupRequiredNameRule {
    pub message: String,
}

impl PageGroupRule for PageGroupRequiredNameRule {
    fn check(&self, item: &PageGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageGroupContactEmailLengthRule {
    pub max_length: i64,
}

impl PageGroupRule for PageGroupContactEmailLengthRule {
    fn check(&self, item: &PageGroup) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageGroupCurrencyLengthRule {
    pub max_length: i64,
}

impl PageGroupRule for PageGroupCurrencyLengthRule {
    fn check(&self, item: &PageGroup) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a page group.
#[derive(Debug, Clone)]
pub struct PageGroupValidator {
    pub rules: Vec<Rc<dyn PageGroupRule>>,
}

impl PageGroupValidator {
    pub fn validate(&self, item: &PageGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_group_validator() -> PageGroupValidator {
    PageGroupValidator {
        rules: vec![
            Rc::new(PageGroupCodeLengthRule { max_length: 120 }) as Rc<dyn PageGroupRule>,
            Rc::new(PageGroupRequiredNameRule {
                message: "page group needs a value".to_string(),
            }),
            Rc::new(PageGroupContactEmailLengthRule { max_length: 255 }),
            Rc::new(PageGroupCurrencyLengthRule { max_length: 32 }),
        ],
    }
}
