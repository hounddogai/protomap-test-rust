use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient nonce record must pass. Check returns its problems, or none.
pub trait LenientNonceRecordRule: std::fmt::Debug {
    fn check(&self, item: &LenientNonceRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceRecordRequiredNameRule {
    pub message: String,
}

impl LenientNonceRecordRule for LenientNonceRecordRequiredNameRule {
    fn check(&self, item: &LenientNonceRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceRecordNameLengthRule {
    pub max_length: i64,
}

impl LenientNonceRecordRule for LenientNonceRecordNameLengthRule {
    fn check(&self, item: &LenientNonceRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceRecordPhoneLengthRule {
    pub max_length: i64,
}

impl LenientNonceRecordRule for LenientNonceRecordPhoneLengthRule {
    fn check(&self, item: &LenientNonceRecord) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient nonce record.
#[derive(Debug, Clone)]
pub struct LenientNonceRecordValidator {
    pub rules: Vec<Rc<dyn LenientNonceRecordRule>>,
}

impl LenientNonceRecordValidator {
    pub fn validate(&self, item: &LenientNonceRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientNonceRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientNonceRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_nonce_record_validator() -> LenientNonceRecordValidator {
    LenientNonceRecordValidator {
        rules: vec![
            Rc::new(LenientNonceRecordRequiredNameRule {
                message: "lenient nonce record needs an entry".to_string(),
            }) as Rc<dyn LenientNonceRecordRule>,
            Rc::new(LenientNonceRecordNameLengthRule { max_length: 120 }),
            Rc::new(LenientNonceRecordPhoneLengthRule { max_length: 64 }),
        ],
    }
}
