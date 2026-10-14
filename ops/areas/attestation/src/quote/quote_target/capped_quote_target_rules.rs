use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped quote target must pass. Check returns its problems, or none.
pub trait CappedQuoteTargetRule: std::fmt::Debug {
    fn check(&self, item: &CappedQuoteTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteTargetRequiredOwnerIdRule {
    pub message: String,
}

impl CappedQuoteTargetRule for CappedQuoteTargetRequiredOwnerIdRule {
    fn check(&self, item: &CappedQuoteTarget) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteTargetAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl CappedQuoteTargetRule for CappedQuoteTargetAllowedOwnerIdRule {
    fn check(&self, item: &CappedQuoteTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is not allowed", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteTargetCodePrefixRule {
    pub prefix: String,
}

impl CappedQuoteTargetRule for CappedQuoteTargetCodePrefixRule {
    fn check(&self, item: &CappedQuoteTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteTargetOwnerIdLengthRule {
    pub max_length: i64,
}

impl CappedQuoteTargetRule for CappedQuoteTargetOwnerIdLengthRule {
    fn check(&self, item: &CappedQuoteTarget) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped quote target.
#[derive(Debug, Clone)]
pub struct CappedQuoteTargetValidator {
    pub rules: Vec<Rc<dyn CappedQuoteTargetRule>>,
}

impl CappedQuoteTargetValidator {
    pub fn validate(&self, item: &CappedQuoteTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedQuoteTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedQuoteTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_quote_target_validator() -> CappedQuoteTargetValidator {
    CappedQuoteTargetValidator {
        rules: vec![
            Rc::new(CappedQuoteTargetRequiredOwnerIdRule {
                message: "capped quote target needs an entry".to_string(),
            }) as Rc<dyn CappedQuoteTargetRule>,
            Rc::new(CappedQuoteTargetAllowedOwnerIdRule {
                allowed: vec!["primary".to_string(), "capped".to_string()],
            }),
            Rc::new(CappedQuoteTargetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CappedQuoteTargetOwnerIdLengthRule { max_length: 16 }),
        ],
    }
}
