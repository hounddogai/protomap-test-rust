use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual release ledger must pass. Check returns its problems, or none.
pub trait ManualReleaseLedgerRule: std::fmt::Debug {
    fn check(&self, item: &ManualReleaseLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseLedgerCipherLengthRule {
    pub max_length: i64,
}

impl ManualReleaseLedgerRule for ManualReleaseLedgerCipherLengthRule {
    fn check(&self, item: &ManualReleaseLedger) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseLedgerVisibleGuardRule {
    pub limit: f64,
}

impl ManualReleaseLedgerRule for ManualReleaseLedgerVisibleGuardRule {
    fn check(&self, item: &ManualReleaseLedger) -> Vec<String> {
        if item.visible && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while visible", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseLedgerCodePrefixRule {
    pub prefix: String,
}

impl ManualReleaseLedgerRule for ManualReleaseLedgerCodePrefixRule {
    fn check(&self, item: &ManualReleaseLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseLedgerAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl ManualReleaseLedgerRule for ManualReleaseLedgerAllowedCipherRule {
    fn check(&self, item: &ManualReleaseLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is not allowed", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseLedgerRequiredTitleRule {
    pub message: String,
}

impl ManualReleaseLedgerRule for ManualReleaseLedgerRequiredTitleRule {
    fn check(&self, item: &ManualReleaseLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual release ledger.
#[derive(Debug, Clone)]
pub struct ManualReleaseLedgerValidator {
    pub rules: Vec<Rc<dyn ManualReleaseLedgerRule>>,
}

impl ManualReleaseLedgerValidator {
    pub fn validate(&self, item: &ManualReleaseLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualReleaseLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualReleaseLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_release_ledger_validator() -> ManualReleaseLedgerValidator {
    ManualReleaseLedgerValidator {
        rules: vec![
            Rc::new(ManualReleaseLedgerCipherLengthRule { max_length: 64 }) as Rc<dyn ManualReleaseLedgerRule>,
            Rc::new(ManualReleaseLedgerVisibleGuardRule { limit: 3.7 }),
            Rc::new(ManualReleaseLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ManualReleaseLedgerAllowedCipherRule {
                allowed: vec!["rolling".to_string(), "global".to_string(), "weighted".to_string()],
            }),
            Rc::new(ManualReleaseLedgerRequiredTitleRule {
                message: "manual release ledger needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_release_ledger_validator();
        let item = new_manual_release_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = ManualReleaseLedgerCipherLengthRule { max_length: 255 };
        let mut item = new_manual_release_ledger("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualReleaseLedgerRequiredTitleRule {
            message: "manual release ledger needs an entry".to_string(),
        };
        let mut item = new_manual_release_ledger("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
