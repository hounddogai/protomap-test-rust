use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration rule must pass. Check returns its problems, or none.
pub trait MigrationRuleRule: std::fmt::Debug {
    fn check(&self, item: &MigrationRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRuleRequiredCipherRule {
    pub message: String,
}

impl MigrationRuleRule for MigrationRuleRequiredCipherRule {
    fn check(&self, item: &MigrationRule) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRuleCodeLengthRule {
    pub max_length: i64,
}

impl MigrationRuleRule for MigrationRuleCodeLengthRule {
    fn check(&self, item: &MigrationRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRuleCodePrefixRule {
    pub prefix: String,
}

impl MigrationRuleRule for MigrationRuleCodePrefixRule {
    fn check(&self, item: &MigrationRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRuleTitleLengthRule {
    pub max_length: i64,
}

impl MigrationRuleRule for MigrationRuleTitleLengthRule {
    fn check(&self, item: &MigrationRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration rule.
#[derive(Debug, Clone)]
pub struct MigrationRuleValidator {
    pub rules: Vec<Rc<dyn MigrationRuleRule>>,
}

impl MigrationRuleValidator {
    pub fn validate(&self, item: &MigrationRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_rule_validator() -> MigrationRuleValidator {
    MigrationRuleValidator {
        rules: vec![
            Rc::new(MigrationRuleRequiredCipherRule {
                message: "migration rule needs an entry".to_string(),
            }) as Rc<dyn MigrationRuleRule>,
            Rc::new(MigrationRuleCodeLengthRule { max_length: 32 }),
            Rc::new(MigrationRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(MigrationRuleTitleLengthRule { max_length: 80 }),
        ],
    }
}
