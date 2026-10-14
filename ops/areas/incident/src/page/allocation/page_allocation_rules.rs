use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page allocation must pass. Check returns its problems, or none.
pub trait PageAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PageAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAllocationRequiredCodeRule {
    pub message: String,
}

impl PageAllocationRule for PageAllocationRequiredCodeRule {
    fn check(&self, item: &PageAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAllocationAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl PageAllocationRule for PageAllocationAllowedScopeRule {
    fn check(&self, item: &PageAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is stale", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAllocationRequiredPhoneRule {
    pub message: String,
}

impl PageAllocationRule for PageAllocationRequiredPhoneRule {
    fn check(&self, item: &PageAllocation) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page allocation.
#[derive(Debug, Clone)]
pub struct PageAllocationValidator {
    pub rules: Vec<Rc<dyn PageAllocationRule>>,
}

impl PageAllocationValidator {
    pub fn validate(&self, item: &PageAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_allocation_validator() -> PageAllocationValidator {
    PageAllocationValidator {
        rules: vec![
            Rc::new(PageAllocationRequiredCodeRule {
                message: "page allocation needs a value".to_string(),
            }) as Rc<dyn PageAllocationRule>,
            Rc::new(PageAllocationAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(PageAllocationRequiredPhoneRule {
                message: "page allocation needs an entry".to_string(),
            }),
        ],
    }
}
