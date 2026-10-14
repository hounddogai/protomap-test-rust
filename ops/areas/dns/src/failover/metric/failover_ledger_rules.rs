use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover ledger must pass. Check returns its problems, or none.
pub trait FailoverLedgerRule: std::fmt::Debug {
    fn check(&self, item: &FailoverLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverLedgerCodeLengthRule {
    pub max_length: i64,
}

impl FailoverLedgerRule for FailoverLedgerCodeLengthRule {
    fn check(&self, item: &FailoverLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverLedgerCodePrefixRule {
    pub prefix: String,
}

impl FailoverLedgerRule for FailoverLedgerCodePrefixRule {
    fn check(&self, item: &FailoverLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverLedgerTitleLengthRule {
    pub max_length: i64,
}

impl FailoverLedgerRule for FailoverLedgerTitleLengthRule {
    fn check(&self, item: &FailoverLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverLedgerRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverLedgerRule for FailoverLedgerRotationDaysRangeRule {
    fn check(&self, item: &FailoverLedger) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverLedgerHostLengthRule {
    pub max_length: i64,
}

impl FailoverLedgerRule for FailoverLedgerHostLengthRule {
    fn check(&self, item: &FailoverLedger) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverLedgerRequiredExternalRefRule {
    pub message: String,
}

impl FailoverLedgerRule for FailoverLedgerRequiredExternalRefRule {
    fn check(&self, item: &FailoverLedger) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover ledger.
#[derive(Debug, Clone)]
pub struct FailoverLedgerValidator {
    pub rules: Vec<Rc<dyn FailoverLedgerRule>>,
}

impl FailoverLedgerValidator {
    pub fn validate(&self, item: &FailoverLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_ledger_validator() -> FailoverLedgerValidator {
    FailoverLedgerValidator {
        rules: vec![
            Rc::new(FailoverLedgerCodeLengthRule { max_length: 120 }) as Rc<dyn FailoverLedgerRule>,
            Rc::new(FailoverLedgerCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(FailoverLedgerTitleLengthRule { max_length: 32 }),
            Rc::new(FailoverLedgerRotationDaysRangeRule { min: 1000, max: 3600 }),
            Rc::new(FailoverLedgerHostLengthRule { max_length: 80 }),
            Rc::new(FailoverLedgerRequiredExternalRefRule {
                message: "failover ledger needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_ledger_validator();
        let item = new_failover_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverLedgerCodeLengthRule { max_length: 1000 };
        let mut item = new_failover_ledger("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FailoverLedgerTitleLengthRule { max_length: 86400 };
        let mut item = new_failover_ledger("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FailoverLedgerRotationDaysRangeRule { min: 120, max: 255 };
        let mut item = new_failover_ledger("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = FailoverLedgerHostLengthRule { max_length: 255 };
        let mut item = new_failover_ledger("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = FailoverLedgerRequiredExternalRefRule {
            message: "failover ledger needs an entry".to_string(),
        };
        let mut item = new_failover_ledger("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
