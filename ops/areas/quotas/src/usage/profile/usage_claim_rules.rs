use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage claim must pass. Check returns its problems, or none.
pub trait UsageClaimRule: std::fmt::Debug {
    fn check(&self, item: &UsageClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageClaimRequiredCurrencyRule {
    pub message: String,
}

impl UsageClaimRule for UsageClaimRequiredCurrencyRule {
    fn check(&self, item: &UsageClaim) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageClaimAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageClaimRule for UsageClaimAmountRangeRule {
    fn check(&self, item: &UsageClaim) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageClaimRequiredCodeRule {
    pub message: String,
}

impl UsageClaimRule for UsageClaimRequiredCodeRule {
    fn check(&self, item: &UsageClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageClaimUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageClaimRule for UsageClaimUpdatedAtRangeRule {
    fn check(&self, item: &UsageClaim) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageClaimArchivedGuardRule {
    pub limit: i64,
}

impl UsageClaimRule for UsageClaimArchivedGuardRule {
    fn check(&self, item: &UsageClaim) -> Vec<String> {
        if item.archived && (item.version > self.limit) {
            return vec![format!(
                "version {} exceeds {} while archived",
                item.version, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageClaimCodeLengthRule {
    pub max_length: i64,
}

impl UsageClaimRule for UsageClaimCodeLengthRule {
    fn check(&self, item: &UsageClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage claim.
#[derive(Debug, Clone)]
pub struct UsageClaimValidator {
    pub rules: Vec<Rc<dyn UsageClaimRule>>,
}

impl UsageClaimValidator {
    pub fn validate(&self, item: &UsageClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_claim_validator() -> UsageClaimValidator {
    UsageClaimValidator {
        rules: vec![
            Rc::new(UsageClaimRequiredCurrencyRule {
                message: "usage claim needs this field".to_string(),
            }) as Rc<dyn UsageClaimRule>,
            Rc::new(UsageClaimAmountRangeRule { min: 5.3, max: 1358.3 }),
            Rc::new(UsageClaimRequiredCodeRule {
                message: "usage claim needs this field".to_string(),
            }),
            Rc::new(UsageClaimUpdatedAtRangeRule { min: 120, max: 86400 }),
            Rc::new(UsageClaimArchivedGuardRule { limit: 3600 }),
            Rc::new(UsageClaimCodeLengthRule { max_length: 3600 }),
        ],
    }
}
