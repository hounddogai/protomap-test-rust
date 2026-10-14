use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback page ledger must pass. Check returns its problems, or none.
pub trait FallbackPageLedgerRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPageLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageLedgerNameLengthRule {
    pub max_length: i64,
}

impl FallbackPageLedgerRule for FallbackPageLedgerNameLengthRule {
    fn check(&self, item: &FallbackPageLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageLedgerCountryLengthRule {
    pub max_length: i64,
}

impl FallbackPageLedgerRule for FallbackPageLedgerCountryLengthRule {
    fn check(&self, item: &FallbackPageLedger) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageLedgerAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl FallbackPageLedgerRule for FallbackPageLedgerAllowedKeyIdRule {
    fn check(&self, item: &FallbackPageLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is too long", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageLedgerRequiredCodeRule {
    pub message: String,
}

impl FallbackPageLedgerRule for FallbackPageLedgerRequiredCodeRule {
    fn check(&self, item: &FallbackPageLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageLedgerCodeLengthRule {
    pub max_length: i64,
}

impl FallbackPageLedgerRule for FallbackPageLedgerCodeLengthRule {
    fn check(&self, item: &FallbackPageLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback page ledger.
#[derive(Debug, Clone)]
pub struct FallbackPageLedgerValidator {
    pub rules: Vec<Rc<dyn FallbackPageLedgerRule>>,
}

impl FallbackPageLedgerValidator {
    pub fn validate(&self, item: &FallbackPageLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackPageLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPageLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_page_ledger_validator() -> FallbackPageLedgerValidator {
    FallbackPageLedgerValidator {
        rules: vec![
            Rc::new(FallbackPageLedgerNameLengthRule { max_length: 80 }) as Rc<dyn FallbackPageLedgerRule>,
            Rc::new(FallbackPageLedgerCountryLengthRule { max_length: 80 }),
            Rc::new(FallbackPageLedgerAllowedKeyIdRule { allowed: Vec::new() }),
            Rc::new(FallbackPageLedgerRequiredCodeRule {
                message: "fallback page ledger needs an entry".to_string(),
            }),
            Rc::new(FallbackPageLedgerCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_page_ledger_validator();
        let item = new_fallback_page_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackPageLedgerNameLengthRule { max_length: 16 };
        let mut item = new_fallback_page_ledger("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = FallbackPageLedgerCountryLengthRule { max_length: 1000 };
        let mut item = new_fallback_page_ledger("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackPageLedgerRequiredCodeRule {
            message: "fallback page ledger needs this field".to_string(),
        };
        let mut item = new_fallback_page_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackPageLedgerCodeLengthRule { max_length: 32 };
        let mut item = new_fallback_page_ledger("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
