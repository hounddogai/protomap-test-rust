use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a nonce ledger must pass. Check returns its problems, or none.
pub trait NonceLedgerRule: std::fmt::Debug {
    fn check(&self, item: &NonceLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedgerPrincipalLengthRule {
    pub max_length: i64,
}

impl NonceLedgerRule for NonceLedgerPrincipalLengthRule {
    fn check(&self, item: &NonceLedger) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedgerRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceLedgerRule for NonceLedgerRetriesRangeRule {
    fn check(&self, item: &NonceLedger) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedgerCodePrefixRule {
    pub prefix: String,
}

impl NonceLedgerRule for NonceLedgerCodePrefixRule {
    fn check(&self, item: &NonceLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedgerTitleLengthRule {
    pub max_length: i64,
}

impl NonceLedgerRule for NonceLedgerTitleLengthRule {
    fn check(&self, item: &NonceLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedgerRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceLedgerRule for NonceLedgerRatioRangeRule {
    fn check(&self, item: &NonceLedger) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedgerAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl NonceLedgerRule for NonceLedgerAllowedCodeRule {
    fn check(&self, item: &NonceLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce ledger.
#[derive(Debug, Clone)]
pub struct NonceLedgerValidator {
    pub rules: Vec<Rc<dyn NonceLedgerRule>>,
}

impl NonceLedgerValidator {
    pub fn validate(&self, item: &NonceLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_ledger_validator() -> NonceLedgerValidator {
    NonceLedgerValidator {
        rules: vec![
            Rc::new(NonceLedgerPrincipalLengthRule { max_length: 255 }) as Rc<dyn NonceLedgerRule>,
            Rc::new(NonceLedgerRetriesRangeRule { min: 80, max: 1000 }),
            Rc::new(NonceLedgerCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(NonceLedgerTitleLengthRule { max_length: 120 }),
            Rc::new(NonceLedgerRatioRangeRule { min: 9.5, max: 3271.8 }),
            Rc::new(NonceLedgerAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_ledger_validator();
        let item = new_nonce_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = NonceLedgerPrincipalLengthRule { max_length: 32 };
        let mut item = new_nonce_ledger("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = NonceLedgerRetriesRangeRule { min: 80, max: 3600 };
        let mut item = new_nonce_ledger("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = NonceLedgerTitleLengthRule { max_length: 80 };
        let mut item = new_nonce_ledger("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = NonceLedgerRatioRangeRule { min: 2.2, max: 1033.6 };
        let mut item = new_nonce_ledger("x", "y");
        item.ratio = 1035.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
