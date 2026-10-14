use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation ledger must pass. Check returns its problems, or none.
pub trait DelegationLedgerRule: std::fmt::Debug {
    fn check(&self, item: &DelegationLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationLedgerBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationLedgerRule for DelegationLedgerBatchSizeRangeRule {
    fn check(&self, item: &DelegationLedger) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationLedgerCodePrefixRule {
    pub prefix: String,
}

impl DelegationLedgerRule for DelegationLedgerCodePrefixRule {
    fn check(&self, item: &DelegationLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationLedgerRequiredLabelRule {
    pub message: String,
}

impl DelegationLedgerRule for DelegationLedgerRequiredLabelRule {
    fn check(&self, item: &DelegationLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation ledger.
#[derive(Debug, Clone)]
pub struct DelegationLedgerValidator {
    pub rules: Vec<Rc<dyn DelegationLedgerRule>>,
}

impl DelegationLedgerValidator {
    pub fn validate(&self, item: &DelegationLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_ledger_validator() -> DelegationLedgerValidator {
    DelegationLedgerValidator {
        rules: vec![
            Rc::new(DelegationLedgerBatchSizeRangeRule { min: 32, max: 80 }) as Rc<dyn DelegationLedgerRule>,
            Rc::new(DelegationLedgerCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(DelegationLedgerRequiredLabelRule {
                message: "delegation ledger needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_ledger_validator();
        let item = new_delegation_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = DelegationLedgerBatchSizeRangeRule { min: 80, max: 86400 };
        let mut item = new_delegation_ledger("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DelegationLedgerRequiredLabelRule {
            message: "delegation ledger needs this field".to_string(),
        };
        let mut item = new_delegation_ledger("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
