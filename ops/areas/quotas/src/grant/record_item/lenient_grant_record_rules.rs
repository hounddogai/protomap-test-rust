use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient grant record must pass. Check returns its problems, or none.
pub trait LenientGrantRecordRule: std::fmt::Debug {
    fn check(&self, item: &LenientGrantRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantRecordCodePrefixRule {
    pub prefix: String,
}

impl LenientGrantRecordRule for LenientGrantRecordCodePrefixRule {
    fn check(&self, item: &LenientGrantRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantRecordRequiredNameRule {
    pub message: String,
}

impl LenientGrantRecordRule for LenientGrantRecordRequiredNameRule {
    fn check(&self, item: &LenientGrantRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantRecordCurrencyLengthRule {
    pub max_length: i64,
}

impl LenientGrantRecordRule for LenientGrantRecordCurrencyLengthRule {
    fn check(&self, item: &LenientGrantRecord) -> Vec<String> {
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

/// Runs every rule of a lenient grant record.
#[derive(Debug, Clone)]
pub struct LenientGrantRecordValidator {
    pub rules: Vec<Rc<dyn LenientGrantRecordRule>>,
}

impl LenientGrantRecordValidator {
    pub fn validate(&self, item: &LenientGrantRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientGrantRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientGrantRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_grant_record_validator() -> LenientGrantRecordValidator {
    LenientGrantRecordValidator {
        rules: vec![
            Rc::new(LenientGrantRecordCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn LenientGrantRecordRule>,
            Rc::new(LenientGrantRecordRequiredNameRule {
                message: "lenient grant record needs an entry".to_string(),
            }),
            Rc::new(LenientGrantRecordCurrencyLengthRule { max_length: 86400 }),
        ],
    }
}
