use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page ledger must pass. Check returns its problems, or none.
pub trait PageLedgerRule: std::fmt::Debug {
    fn check(&self, item: &PageLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLedgerCodeLengthRule {
    pub max_length: i64,
}

impl PageLedgerRule for PageLedgerCodeLengthRule {
    fn check(&self, item: &PageLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLedgerRequiredNamespaceRule {
    pub message: String,
}

impl PageLedgerRule for PageLedgerRequiredNamespaceRule {
    fn check(&self, item: &PageLedger) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLedgerLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageLedgerRule for PageLedgerLimitRangeRule {
    fn check(&self, item: &PageLedger) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLedgerCountryLengthRule {
    pub max_length: i64,
}

impl PageLedgerRule for PageLedgerCountryLengthRule {
    fn check(&self, item: &PageLedger) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLedgerRequiredCodeRule {
    pub message: String,
}

impl PageLedgerRule for PageLedgerRequiredCodeRule {
    fn check(&self, item: &PageLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLedgerNamespaceLengthRule {
    pub max_length: i64,
}

impl PageLedgerRule for PageLedgerNamespaceLengthRule {
    fn check(&self, item: &PageLedger) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a page ledger.
#[derive(Debug, Clone)]
pub struct PageLedgerValidator {
    pub rules: Vec<Rc<dyn PageLedgerRule>>,
}

impl PageLedgerValidator {
    pub fn validate(&self, item: &PageLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_ledger_validator() -> PageLedgerValidator {
    PageLedgerValidator {
        rules: vec![
            Rc::new(PageLedgerCodeLengthRule { max_length: 80 }) as Rc<dyn PageLedgerRule>,
            Rc::new(PageLedgerRequiredNamespaceRule {
                message: "page ledger needs an entry".to_string(),
            }),
            Rc::new(PageLedgerLimitRangeRule { min: 32, max: 255 }),
            Rc::new(PageLedgerCountryLengthRule { max_length: 1000 }),
            Rc::new(PageLedgerRequiredCodeRule {
                message: "page ledger needs an entry".to_string(),
            }),
            Rc::new(PageLedgerNamespaceLengthRule { max_length: 16 }),
        ],
    }
}
