use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page tier must pass. Check returns its problems, or none.
pub trait PageTierRule: std::fmt::Debug {
    fn check(&self, item: &PageTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTierRequiredOwnerIdRule {
    pub message: String,
}

impl PageTierRule for PageTierRequiredOwnerIdRule {
    fn check(&self, item: &PageTier) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTierNameLengthRule {
    pub max_length: i64,
}

impl PageTierRule for PageTierNameLengthRule {
    fn check(&self, item: &PageTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTierCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageTierRule for PageTierCapacityRangeRule {
    fn check(&self, item: &PageTier) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page tier.
#[derive(Debug, Clone)]
pub struct PageTierValidator {
    pub rules: Vec<Rc<dyn PageTierRule>>,
}

impl PageTierValidator {
    pub fn validate(&self, item: &PageTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_tier_validator() -> PageTierValidator {
    PageTierValidator {
        rules: vec![
            Rc::new(PageTierRequiredOwnerIdRule {
                message: "page tier needs this field".to_string(),
            }) as Rc<dyn PageTierRule>,
            Rc::new(PageTierNameLengthRule { max_length: 3600 }),
            Rc::new(PageTierCapacityRangeRule { min: 32, max: 64 }),
        ],
    }
}
