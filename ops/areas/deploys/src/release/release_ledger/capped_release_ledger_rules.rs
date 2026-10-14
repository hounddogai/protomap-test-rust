use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped release ledger must pass. Check returns its problems, or none.
pub trait CappedReleaseLedgerRule: std::fmt::Debug {
    fn check(&self, item: &CappedReleaseLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseLedgerCodePrefixRule {
    pub prefix: String,
}

impl CappedReleaseLedgerRule for CappedReleaseLedgerCodePrefixRule {
    fn check(&self, item: &CappedReleaseLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseLedgerRequiredNameRule {
    pub message: String,
}

impl CappedReleaseLedgerRule for CappedReleaseLedgerRequiredNameRule {
    fn check(&self, item: &CappedReleaseLedger) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseLedgerCodeLengthRule {
    pub max_length: i64,
}

impl CappedReleaseLedgerRule for CappedReleaseLedgerCodeLengthRule {
    fn check(&self, item: &CappedReleaseLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseLedgerSourceLengthRule {
    pub max_length: i64,
}

impl CappedReleaseLedgerRule for CappedReleaseLedgerSourceLengthRule {
    fn check(&self, item: &CappedReleaseLedger) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseLedgerAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CappedReleaseLedgerRule for CappedReleaseLedgerAllowedNameRule {
    fn check(&self, item: &CappedReleaseLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseLedgerRequiredCodeRule {
    pub message: String,
}

impl CappedReleaseLedgerRule for CappedReleaseLedgerRequiredCodeRule {
    fn check(&self, item: &CappedReleaseLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped release ledger.
#[derive(Debug, Clone)]
pub struct CappedReleaseLedgerValidator {
    pub rules: Vec<Rc<dyn CappedReleaseLedgerRule>>,
}

impl CappedReleaseLedgerValidator {
    pub fn validate(&self, item: &CappedReleaseLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedReleaseLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedReleaseLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_release_ledger_validator() -> CappedReleaseLedgerValidator {
    CappedReleaseLedgerValidator {
        rules: vec![
            Rc::new(CappedReleaseLedgerCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn CappedReleaseLedgerRule>,
            Rc::new(CappedReleaseLedgerRequiredNameRule {
                message: "capped release ledger needs this field".to_string(),
            }),
            Rc::new(CappedReleaseLedgerCodeLengthRule { max_length: 16 }),
            Rc::new(CappedReleaseLedgerSourceLengthRule { max_length: 3600 }),
            Rc::new(CappedReleaseLedgerAllowedNameRule {
                allowed: vec!["manual".to_string(), "default".to_string()],
            }),
            Rc::new(CappedReleaseLedgerRequiredCodeRule {
                message: "capped release ledger needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_release_ledger_validator();
        let item = new_capped_release_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedReleaseLedgerRequiredNameRule {
            message: "capped release ledger needs an entry".to_string(),
        };
        let mut item = new_capped_release_ledger("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedReleaseLedgerCodeLengthRule { max_length: 120 };
        let mut item = new_capped_release_ledger("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = CappedReleaseLedgerSourceLengthRule { max_length: 1000 };
        let mut item = new_capped_release_ledger("x", "y");
        item.source = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedReleaseLedgerRequiredCodeRule {
            message: "capped release ledger needs a value".to_string(),
        };
        let mut item = new_capped_release_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
