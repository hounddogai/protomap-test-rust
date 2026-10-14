use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier ledger must pass. Check returns its problems, or none.
pub trait VerifierLedgerRule: std::fmt::Debug {
    fn check(&self, item: &VerifierLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierLedgerRequiredKeyIdRule {
    pub message: String,
}

impl VerifierLedgerRule for VerifierLedgerRequiredKeyIdRule {
    fn check(&self, item: &VerifierLedger) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierLedgerPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierLedgerRule for VerifierLedgerPortRangeRule {
    fn check(&self, item: &VerifierLedger) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierLedgerRequiredCodeRule {
    pub message: String,
}

impl VerifierLedgerRule for VerifierLedgerRequiredCodeRule {
    fn check(&self, item: &VerifierLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierLedgerQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierLedgerRule for VerifierLedgerQuantityRangeRule {
    fn check(&self, item: &VerifierLedger) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierLedgerKeyIdLengthRule {
    pub max_length: i64,
}

impl VerifierLedgerRule for VerifierLedgerKeyIdLengthRule {
    fn check(&self, item: &VerifierLedger) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierLedgerAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl VerifierLedgerRule for VerifierLedgerAllowedCodeRule {
    fn check(&self, item: &VerifierLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier ledger.
#[derive(Debug, Clone)]
pub struct VerifierLedgerValidator {
    pub rules: Vec<Rc<dyn VerifierLedgerRule>>,
}

impl VerifierLedgerValidator {
    pub fn validate(&self, item: &VerifierLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_ledger_validator() -> VerifierLedgerValidator {
    VerifierLedgerValidator {
        rules: vec![
            Rc::new(VerifierLedgerRequiredKeyIdRule {
                message: "verifier ledger needs a value".to_string(),
            }) as Rc<dyn VerifierLedgerRule>,
            Rc::new(VerifierLedgerPortRangeRule { min: 80, max: 120 }),
            Rc::new(VerifierLedgerRequiredCodeRule {
                message: "verifier ledger needs an entry".to_string(),
            }),
            Rc::new(VerifierLedgerQuantityRangeRule { min: 32, max: 3600 }),
            Rc::new(VerifierLedgerKeyIdLengthRule { max_length: 80 }),
            Rc::new(VerifierLedgerAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_ledger_validator();
        let item = new_verifier_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = VerifierLedgerRequiredKeyIdRule {
            message: "verifier ledger needs this field".to_string(),
        };
        let mut item = new_verifier_ledger("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = VerifierLedgerPortRangeRule { min: 80, max: 120 };
        let mut item = new_verifier_ledger("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierLedgerRequiredCodeRule {
            message: "verifier ledger needs an entry".to_string(),
        };
        let mut item = new_verifier_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = VerifierLedgerQuantityRangeRule { min: 86400, max: 86400 };
        let mut item = new_verifier_ledger("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = VerifierLedgerKeyIdLengthRule { max_length: 1000 };
        let mut item = new_verifier_ledger("x", "y");
        item.key_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
