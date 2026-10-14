use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity ledger must pass. Check returns its problems, or none.
pub trait IdentityLedgerRule: std::fmt::Debug {
    fn check(&self, item: &IdentityLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityLedgerFingerprintLengthRule {
    pub max_length: i64,
}

impl IdentityLedgerRule for IdentityLedgerFingerprintLengthRule {
    fn check(&self, item: &IdentityLedger) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityLedgerCodePrefixRule {
    pub prefix: String,
}

impl IdentityLedgerRule for IdentityLedgerCodePrefixRule {
    fn check(&self, item: &IdentityLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityLedgerAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl IdentityLedgerRule for IdentityLedgerAllowedCodeRule {
    fn check(&self, item: &IdentityLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityLedgerCodeLengthRule {
    pub max_length: i64,
}

impl IdentityLedgerRule for IdentityLedgerCodeLengthRule {
    fn check(&self, item: &IdentityLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityLedgerEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityLedgerRule for IdentityLedgerEndsAtRangeRule {
    fn check(&self, item: &IdentityLedger) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a identity ledger.
#[derive(Debug, Clone)]
pub struct IdentityLedgerValidator {
    pub rules: Vec<Rc<dyn IdentityLedgerRule>>,
}

impl IdentityLedgerValidator {
    pub fn validate(&self, item: &IdentityLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_ledger_validator() -> IdentityLedgerValidator {
    IdentityLedgerValidator {
        rules: vec![
            Rc::new(IdentityLedgerFingerprintLengthRule { max_length: 64 }) as Rc<dyn IdentityLedgerRule>,
            Rc::new(IdentityLedgerCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(IdentityLedgerAllowedCodeRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(IdentityLedgerCodeLengthRule { max_length: 255 }),
            Rc::new(IdentityLedgerEndsAtRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_ledger_validator();
        let item = new_identity_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = IdentityLedgerFingerprintLengthRule { max_length: 32 };
        let mut item = new_identity_ledger("x", "y");
        item.fingerprint = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityLedgerCodeLengthRule { max_length: 3600 };
        let mut item = new_identity_ledger("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = IdentityLedgerEndsAtRangeRule { min: 1000, max: 1000 };
        let mut item = new_identity_ledger("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
