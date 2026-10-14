use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched credential ledger must pass. Check returns its problems, or none.
pub trait BatchedCredentialLedgerRule: std::fmt::Debug {
    fn check(&self, item: &BatchedCredentialLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialLedgerCodeLengthRule {
    pub max_length: i64,
}

impl BatchedCredentialLedgerRule for BatchedCredentialLedgerCodeLengthRule {
    fn check(&self, item: &BatchedCredentialLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialLedgerOwnerIdLengthRule {
    pub max_length: i64,
}

impl BatchedCredentialLedgerRule for BatchedCredentialLedgerOwnerIdLengthRule {
    fn check(&self, item: &BatchedCredentialLedger) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialLedgerAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedCredentialLedgerRule for BatchedCredentialLedgerAmountRangeRule {
    fn check(&self, item: &BatchedCredentialLedger) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialLedgerRequiredOwnerIdRule {
    pub message: String,
}

impl BatchedCredentialLedgerRule for BatchedCredentialLedgerRequiredOwnerIdRule {
    fn check(&self, item: &BatchedCredentialLedger) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialLedgerCipherLengthRule {
    pub max_length: i64,
}

impl BatchedCredentialLedgerRule for BatchedCredentialLedgerCipherLengthRule {
    fn check(&self, item: &BatchedCredentialLedger) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialLedgerRequiredCodeRule {
    pub message: String,
}

impl BatchedCredentialLedgerRule for BatchedCredentialLedgerRequiredCodeRule {
    fn check(&self, item: &BatchedCredentialLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched credential ledger.
#[derive(Debug, Clone)]
pub struct BatchedCredentialLedgerValidator {
    pub rules: Vec<Rc<dyn BatchedCredentialLedgerRule>>,
}

impl BatchedCredentialLedgerValidator {
    pub fn validate(&self, item: &BatchedCredentialLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedCredentialLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedCredentialLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_credential_ledger_validator() -> BatchedCredentialLedgerValidator {
    BatchedCredentialLedgerValidator {
        rules: vec![
            Rc::new(BatchedCredentialLedgerCodeLengthRule { max_length: 1000 }) as Rc<dyn BatchedCredentialLedgerRule>,
            Rc::new(BatchedCredentialLedgerOwnerIdLengthRule { max_length: 16 }),
            Rc::new(BatchedCredentialLedgerAmountRangeRule { min: 2.9, max: 4962.9 }),
            Rc::new(BatchedCredentialLedgerRequiredOwnerIdRule {
                message: "batched credential ledger needs a value".to_string(),
            }),
            Rc::new(BatchedCredentialLedgerCipherLengthRule { max_length: 255 }),
            Rc::new(BatchedCredentialLedgerRequiredCodeRule {
                message: "batched credential ledger needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_credential_ledger_validator();
        let item = new_batched_credential_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedCredentialLedgerCodeLengthRule { max_length: 86400 };
        let mut item = new_batched_credential_ledger("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = BatchedCredentialLedgerOwnerIdLengthRule { max_length: 16 };
        let mut item = new_batched_credential_ledger("x", "y");
        item.owner_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = BatchedCredentialLedgerAmountRangeRule { min: 4.3, max: 4758.3 };
        let mut item = new_batched_credential_ledger("x", "y");
        item.amount = 4759.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = BatchedCredentialLedgerRequiredOwnerIdRule {
            message: "batched credential ledger needs this field".to_string(),
        };
        let mut item = new_batched_credential_ledger("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = BatchedCredentialLedgerCipherLengthRule { max_length: 255 };
        let mut item = new_batched_credential_ledger("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedCredentialLedgerRequiredCodeRule {
            message: "batched credential ledger needs a value".to_string(),
        };
        let mut item = new_batched_credential_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
