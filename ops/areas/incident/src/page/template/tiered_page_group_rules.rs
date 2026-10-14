use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered page group must pass. Check returns its problems, or none.
pub trait TieredPageGroupRule: std::fmt::Debug {
    fn check(&self, item: &TieredPageGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroupRequiredHostRule {
    pub message: String,
}

impl TieredPageGroupRule for TieredPageGroupRequiredHostRule {
    fn check(&self, item: &TieredPageGroup) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroupRequiredTitleRule {
    pub message: String,
}

impl TieredPageGroupRule for TieredPageGroupRequiredTitleRule {
    fn check(&self, item: &TieredPageGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroupCodePrefixRule {
    pub prefix: String,
}

impl TieredPageGroupRule for TieredPageGroupCodePrefixRule {
    fn check(&self, item: &TieredPageGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroupBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredPageGroupRule for TieredPageGroupBudgetRangeRule {
    fn check(&self, item: &TieredPageGroup) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroupRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredPageGroupRule for TieredPageGroupRetriesRangeRule {
    fn check(&self, item: &TieredPageGroup) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroupMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredPageGroupRule for TieredPageGroupMarginRangeRule {
    fn check(&self, item: &TieredPageGroup) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered page group.
#[derive(Debug, Clone)]
pub struct TieredPageGroupValidator {
    pub rules: Vec<Rc<dyn TieredPageGroupRule>>,
}

impl TieredPageGroupValidator {
    pub fn validate(&self, item: &TieredPageGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredPageGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPageGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_page_group_validator() -> TieredPageGroupValidator {
    TieredPageGroupValidator {
        rules: vec![
            Rc::new(TieredPageGroupRequiredHostRule {
                message: "tiered page group needs this field".to_string(),
            }) as Rc<dyn TieredPageGroupRule>,
            Rc::new(TieredPageGroupRequiredTitleRule {
                message: "tiered page group needs a value".to_string(),
            }),
            Rc::new(TieredPageGroupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredPageGroupBudgetRangeRule { min: 4.9, max: 555.7 }),
            Rc::new(TieredPageGroupRetriesRangeRule { min: 32, max: 255 }),
            Rc::new(TieredPageGroupMarginRangeRule { min: 4.6, max: 1689.6 }),
        ],
    }
}
