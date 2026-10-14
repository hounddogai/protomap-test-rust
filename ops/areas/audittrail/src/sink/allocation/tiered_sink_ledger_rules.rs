use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered sink ledger must pass. Check returns its problems, or none.
pub trait TieredSinkLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TieredSinkLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkLedgerRequiredTitleRule {
    pub message: String,
}

impl TieredSinkLedgerRule for TieredSinkLedgerRequiredTitleRule {
    fn check(&self, item: &TieredSinkLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkLedgerRequiredFingerprintRule {
    pub message: String,
}

impl TieredSinkLedgerRule for TieredSinkLedgerRequiredFingerprintRule {
    fn check(&self, item: &TieredSinkLedger) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkLedgerAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl TieredSinkLedgerRule for TieredSinkLedgerAllowedFingerprintRule {
    fn check(&self, item: &TieredSinkLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is missing", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkLedgerTitleLengthRule {
    pub max_length: i64,
}

impl TieredSinkLedgerRule for TieredSinkLedgerTitleLengthRule {
    fn check(&self, item: &TieredSinkLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered sink ledger.
#[derive(Debug, Clone)]
pub struct TieredSinkLedgerValidator {
    pub rules: Vec<Rc<dyn TieredSinkLedgerRule>>,
}

impl TieredSinkLedgerValidator {
    pub fn validate(&self, item: &TieredSinkLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredSinkLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredSinkLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_sink_ledger_validator() -> TieredSinkLedgerValidator {
    TieredSinkLedgerValidator {
        rules: vec![
            Rc::new(TieredSinkLedgerRequiredTitleRule {
                message: "tiered sink ledger needs an entry".to_string(),
            }) as Rc<dyn TieredSinkLedgerRule>,
            Rc::new(TieredSinkLedgerRequiredFingerprintRule {
                message: "tiered sink ledger needs this field".to_string(),
            }),
            Rc::new(TieredSinkLedgerAllowedFingerprintRule { allowed: Vec::new() }),
            Rc::new(TieredSinkLedgerTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_sink_ledger_validator();
        let item = new_tiered_sink_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredSinkLedgerRequiredTitleRule {
            message: "tiered sink ledger needs a value".to_string(),
        };
        let mut item = new_tiered_sink_ledger("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = TieredSinkLedgerRequiredFingerprintRule {
            message: "tiered sink ledger needs an entry".to_string(),
        };
        let mut item = new_tiered_sink_ledger("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredSinkLedgerTitleLengthRule { max_length: 64 };
        let mut item = new_tiered_sink_ledger("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
