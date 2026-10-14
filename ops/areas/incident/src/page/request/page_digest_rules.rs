use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page digest must pass. Check returns its problems, or none.
pub trait PageDigestRule: std::fmt::Debug {
    fn check(&self, item: &PageDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageDigestRequiredCodeRule {
    pub message: String,
}

impl PageDigestRule for PageDigestRequiredCodeRule {
    fn check(&self, item: &PageDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageDigestNameLengthRule {
    pub max_length: i64,
}

impl PageDigestRule for PageDigestNameLengthRule {
    fn check(&self, item: &PageDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageDigestRequiredNameRule {
    pub message: String,
}

impl PageDigestRule for PageDigestRequiredNameRule {
    fn check(&self, item: &PageDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page digest.
#[derive(Debug, Clone)]
pub struct PageDigestValidator {
    pub rules: Vec<Rc<dyn PageDigestRule>>,
}

impl PageDigestValidator {
    pub fn validate(&self, item: &PageDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_digest_validator() -> PageDigestValidator {
    PageDigestValidator {
        rules: vec![
            Rc::new(PageDigestRequiredCodeRule {
                message: "page digest needs this field".to_string(),
            }) as Rc<dyn PageDigestRule>,
            Rc::new(PageDigestNameLengthRule { max_length: 64 }),
            Rc::new(PageDigestRequiredNameRule {
                message: "page digest needs an entry".to_string(),
            }),
        ],
    }
}
