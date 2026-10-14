use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback failover budget must pass. Check returns its problems, or none.
pub trait FallbackFailoverBudgetRule: std::fmt::Debug {
    fn check(&self, item: &FallbackFailoverBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverBudgetKeyIdLengthRule {
    pub max_length: i64,
}

impl FallbackFailoverBudgetRule for FallbackFailoverBudgetKeyIdLengthRule {
    fn check(&self, item: &FallbackFailoverBudget) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverBudgetCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackFailoverBudgetRule for FallbackFailoverBudgetCreatedAtRangeRule {
    fn check(&self, item: &FallbackFailoverBudget) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverBudgetCodePrefixRule {
    pub prefix: String,
}

impl FallbackFailoverBudgetRule for FallbackFailoverBudgetCodePrefixRule {
    fn check(&self, item: &FallbackFailoverBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverBudgetRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackFailoverBudgetRule for FallbackFailoverBudgetRateRangeRule {
    fn check(&self, item: &FallbackFailoverBudget) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverBudgetRequiredCodeRule {
    pub message: String,
}

impl FallbackFailoverBudgetRule for FallbackFailoverBudgetRequiredCodeRule {
    fn check(&self, item: &FallbackFailoverBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverBudgetBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackFailoverBudgetRule for FallbackFailoverBudgetBurstRangeRule {
    fn check(&self, item: &FallbackFailoverBudget) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback failover budget.
#[derive(Debug, Clone)]
pub struct FallbackFailoverBudgetValidator {
    pub rules: Vec<Rc<dyn FallbackFailoverBudgetRule>>,
}

impl FallbackFailoverBudgetValidator {
    pub fn validate(&self, item: &FallbackFailoverBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackFailoverBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackFailoverBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_failover_budget_validator() -> FallbackFailoverBudgetValidator {
    FallbackFailoverBudgetValidator {
        rules: vec![
            Rc::new(FallbackFailoverBudgetKeyIdLengthRule { max_length: 1000 }) as Rc<dyn FallbackFailoverBudgetRule>,
            Rc::new(FallbackFailoverBudgetCreatedAtRangeRule { min: 64, max: 1000 }),
            Rc::new(FallbackFailoverBudgetCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(FallbackFailoverBudgetRateRangeRule { min: 4.2, max: 2123.6 }),
            Rc::new(FallbackFailoverBudgetRequiredCodeRule {
                message: "fallback failover budget needs an entry".to_string(),
            }),
            Rc::new(FallbackFailoverBudgetBurstRangeRule { min: 16, max: 80 }),
        ],
    }
}
