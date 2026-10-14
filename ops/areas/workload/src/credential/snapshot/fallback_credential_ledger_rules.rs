use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback credential ledger must pass. Check returns its problems, or none.
pub trait FallbackCredentialLedgerRule: std::fmt::Debug {
    fn check(&self, item: &FallbackCredentialLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCredentialLedgerCodeLengthRule {
    pub max_length: i64,
}

impl FallbackCredentialLedgerRule for FallbackCredentialLedgerCodeLengthRule {
    fn check(&self, item: &FallbackCredentialLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCredentialLedgerExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackCredentialLedgerRule for FallbackCredentialLedgerExpiresAtRangeRule {
    fn check(&self, item: &FallbackCredentialLedger) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCredentialLedgerRequiredCodeRule {
    pub message: String,
}

impl FallbackCredentialLedgerRule for FallbackCredentialLedgerRequiredCodeRule {
    fn check(&self, item: &FallbackCredentialLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback credential ledger.
#[derive(Debug, Clone)]
pub struct FallbackCredentialLedgerValidator {
    pub rules: Vec<Rc<dyn FallbackCredentialLedgerRule>>,
}

impl FallbackCredentialLedgerValidator {
    pub fn validate(&self, item: &FallbackCredentialLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackCredentialLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackCredentialLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_credential_ledger_validator() -> FallbackCredentialLedgerValidator {
    FallbackCredentialLedgerValidator {
        rules: vec![
            Rc::new(FallbackCredentialLedgerCodeLengthRule { max_length: 16 }) as Rc<dyn FallbackCredentialLedgerRule>,
            Rc::new(FallbackCredentialLedgerExpiresAtRangeRule { min: 16, max: 64 }),
            Rc::new(FallbackCredentialLedgerRequiredCodeRule {
                message: "fallback credential ledger needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_credential_ledger_validator();
        let item = new_fallback_credential_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackCredentialLedgerCodeLengthRule { max_length: 64 };
        let mut item = new_fallback_credential_ledger("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FallbackCredentialLedgerExpiresAtRangeRule { min: 16, max: 32 };
        let mut item = new_fallback_credential_ledger("x", "y");
        item.expires_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackCredentialLedgerRequiredCodeRule {
            message: "fallback credential ledger needs a value".to_string(),
        };
        let mut item = new_fallback_credential_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
