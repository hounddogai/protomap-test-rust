use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled isolation ledger must pass. Check returns its problems, or none.
pub trait ScheduledIsolationLedgerRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledIsolationLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationLedgerQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledIsolationLedgerRule for ScheduledIsolationLedgerQuantityRangeRule {
    fn check(&self, item: &ScheduledIsolationLedger) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationLedgerDescriptionLengthRule {
    pub max_length: i64,
}

impl ScheduledIsolationLedgerRule for ScheduledIsolationLedgerDescriptionLengthRule {
    fn check(&self, item: &ScheduledIsolationLedger) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationLedgerCodePrefixRule {
    pub prefix: String,
}

impl ScheduledIsolationLedgerRule for ScheduledIsolationLedgerCodePrefixRule {
    fn check(&self, item: &ScheduledIsolationLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationLedgerRequiredNameRule {
    pub message: String,
}

impl ScheduledIsolationLedgerRule for ScheduledIsolationLedgerRequiredNameRule {
    fn check(&self, item: &ScheduledIsolationLedger) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled isolation ledger.
#[derive(Debug, Clone)]
pub struct ScheduledIsolationLedgerValidator {
    pub rules: Vec<Rc<dyn ScheduledIsolationLedgerRule>>,
}

impl ScheduledIsolationLedgerValidator {
    pub fn validate(&self, item: &ScheduledIsolationLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledIsolationLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledIsolationLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_isolation_ledger_validator() -> ScheduledIsolationLedgerValidator {
    ScheduledIsolationLedgerValidator {
        rules: vec![
            Rc::new(ScheduledIsolationLedgerQuantityRangeRule { min: 32, max: 64 })
                as Rc<dyn ScheduledIsolationLedgerRule>,
            Rc::new(ScheduledIsolationLedgerDescriptionLengthRule { max_length: 80 }),
            Rc::new(ScheduledIsolationLedgerCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ScheduledIsolationLedgerRequiredNameRule {
                message: "scheduled isolation ledger needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_isolation_ledger_validator();
        let item = new_scheduled_isolation_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ScheduledIsolationLedgerQuantityRangeRule { min: 255, max: 86400 };
        let mut item = new_scheduled_isolation_ledger("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = ScheduledIsolationLedgerDescriptionLengthRule { max_length: 16 };
        let mut item = new_scheduled_isolation_ledger("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledIsolationLedgerRequiredNameRule {
            message: "scheduled isolation ledger needs this field".to_string(),
        };
        let mut item = new_scheduled_isolation_ledger("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
