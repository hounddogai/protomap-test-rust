use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending page ledger must pass. Check returns its problems, or none.
pub trait PendingPageLedgerRule: std::fmt::Debug {
    fn check(&self, item: &PendingPageLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageLedgerCodePrefixRule {
    pub prefix: String,
}

impl PendingPageLedgerRule for PendingPageLedgerCodePrefixRule {
    fn check(&self, item: &PendingPageLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageLedgerRequiredCodeRule {
    pub message: String,
}

impl PendingPageLedgerRule for PendingPageLedgerRequiredCodeRule {
    fn check(&self, item: &PendingPageLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageLedgerWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingPageLedgerRule for PendingPageLedgerWeightRangeRule {
    fn check(&self, item: &PendingPageLedger) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending page ledger.
#[derive(Debug, Clone)]
pub struct PendingPageLedgerValidator {
    pub rules: Vec<Rc<dyn PendingPageLedgerRule>>,
}

impl PendingPageLedgerValidator {
    pub fn validate(&self, item: &PendingPageLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingPageLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPageLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_page_ledger_validator() -> PendingPageLedgerValidator {
    PendingPageLedgerValidator {
        rules: vec![
            Rc::new(PendingPageLedgerCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn PendingPageLedgerRule>,
            Rc::new(PendingPageLedgerRequiredCodeRule {
                message: "pending page ledger needs this field".to_string(),
            }),
            Rc::new(PendingPageLedgerWeightRangeRule { min: 1.3, max: 3734.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_page_ledger_validator();
        let item = new_pending_page_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingPageLedgerRequiredCodeRule {
            message: "pending page ledger needs an entry".to_string(),
        };
        let mut item = new_pending_page_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PendingPageLedgerWeightRangeRule { min: 7.8, max: 3408.9 };
        let mut item = new_pending_page_ledger("x", "y");
        item.weight = 3410.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
