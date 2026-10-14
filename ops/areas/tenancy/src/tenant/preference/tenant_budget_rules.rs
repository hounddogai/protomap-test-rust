use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant budget must pass. Check returns its problems, or none.
pub trait TenantBudgetRule: std::fmt::Debug {
    fn check(&self, item: &TenantBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBudgetCodePrefixRule {
    pub prefix: String,
}

impl TenantBudgetRule for TenantBudgetCodePrefixRule {
    fn check(&self, item: &TenantBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBudgetRequiredCodeRule {
    pub message: String,
}

impl TenantBudgetRule for TenantBudgetRequiredCodeRule {
    fn check(&self, item: &TenantBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBudgetWindowRule {
    pub max_span: i64,
}

impl TenantBudgetRule for TenantBudgetWindowRule {
    fn check(&self, item: &TenantBudget) -> Vec<String> {
        if item.starts_at > item.ends_at {
            return vec!["starts at is after ends at".to_string()];
        }
        if self.max_span > 0 && (item.ends_at - item.starts_at > self.max_span) {
            return vec![format!("spans more than {} seconds", self.max_span)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBudgetRequiredTimezoneRule {
    pub message: String,
}

impl TenantBudgetRule for TenantBudgetRequiredTimezoneRule {
    fn check(&self, item: &TenantBudget) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBudgetTimezoneLengthRule {
    pub max_length: i64,
}

impl TenantBudgetRule for TenantBudgetTimezoneLengthRule {
    fn check(&self, item: &TenantBudget) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBudgetDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantBudgetRule for TenantBudgetDiscountRangeRule {
    fn check(&self, item: &TenantBudget) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tenant budget.
#[derive(Debug, Clone)]
pub struct TenantBudgetValidator {
    pub rules: Vec<Rc<dyn TenantBudgetRule>>,
}

impl TenantBudgetValidator {
    pub fn validate(&self, item: &TenantBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_budget_validator() -> TenantBudgetValidator {
    TenantBudgetValidator {
        rules: vec![
            Rc::new(TenantBudgetCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn TenantBudgetRule>,
            Rc::new(TenantBudgetRequiredCodeRule {
                message: "tenant budget needs a value".to_string(),
            }),
            Rc::new(TenantBudgetWindowRule { max_span: 120 }),
            Rc::new(TenantBudgetRequiredTimezoneRule {
                message: "tenant budget needs this field".to_string(),
            }),
            Rc::new(TenantBudgetTimezoneLengthRule { max_length: 32 }),
            Rc::new(TenantBudgetDiscountRangeRule { min: 9.7, max: 73.3 }),
        ],
    }
}
