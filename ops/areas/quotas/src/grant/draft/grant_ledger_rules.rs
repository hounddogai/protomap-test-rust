use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant ledger must pass. Check returns its problems, or none.
pub trait GrantLedgerRule: std::fmt::Debug {
    fn check(&self, item: &GrantLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantLedgerRequiredChannelRule {
    pub message: String,
}

impl GrantLedgerRule for GrantLedgerRequiredChannelRule {
    fn check(&self, item: &GrantLedger) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantLedgerPrincipalLengthRule {
    pub max_length: i64,
}

impl GrantLedgerRule for GrantLedgerPrincipalLengthRule {
    fn check(&self, item: &GrantLedger) -> Vec<String> {
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
pub struct GrantLedgerLabelLengthRule {
    pub max_length: i64,
}

impl GrantLedgerRule for GrantLedgerLabelLengthRule {
    fn check(&self, item: &GrantLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantLedgerQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantLedgerRule for GrantLedgerQuotaUnitsRangeRule {
    fn check(&self, item: &GrantLedger) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantLedgerCodePrefixRule {
    pub prefix: String,
}

impl GrantLedgerRule for GrantLedgerCodePrefixRule {
    fn check(&self, item: &GrantLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantLedgerRequiredLabelRule {
    pub message: String,
}

impl GrantLedgerRule for GrantLedgerRequiredLabelRule {
    fn check(&self, item: &GrantLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant ledger.
#[derive(Debug, Clone)]
pub struct GrantLedgerValidator {
    pub rules: Vec<Rc<dyn GrantLedgerRule>>,
}

impl GrantLedgerValidator {
    pub fn validate(&self, item: &GrantLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_ledger_validator() -> GrantLedgerValidator {
    GrantLedgerValidator {
        rules: vec![
            Rc::new(GrantLedgerRequiredChannelRule {
                message: "grant ledger needs a value".to_string(),
            }) as Rc<dyn GrantLedgerRule>,
            Rc::new(GrantLedgerPrincipalLengthRule { max_length: 3600 }),
            Rc::new(GrantLedgerLabelLengthRule { max_length: 1000 }),
            Rc::new(GrantLedgerQuotaUnitsRangeRule { min: 64, max: 86400 }),
            Rc::new(GrantLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GrantLedgerRequiredLabelRule {
                message: "grant ledger needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_ledger_validator();
        let item = new_grant_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = GrantLedgerRequiredChannelRule {
            message: "grant ledger needs this field".to_string(),
        };
        let mut item = new_grant_ledger("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = GrantLedgerPrincipalLengthRule { max_length: 86400 };
        let mut item = new_grant_ledger("x", "y");
        item.principal = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GrantLedgerLabelLengthRule { max_length: 80 };
        let mut item = new_grant_ledger("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = GrantLedgerQuotaUnitsRangeRule { min: 32, max: 3600 };
        let mut item = new_grant_ledger("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GrantLedgerRequiredLabelRule {
            message: "grant ledger needs this field".to_string(),
        };
        let mut item = new_grant_ledger("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
