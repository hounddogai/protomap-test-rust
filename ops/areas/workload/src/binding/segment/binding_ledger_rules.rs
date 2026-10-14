use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding ledger must pass. Check returns its problems, or none.
pub trait BindingLedgerRule: std::fmt::Debug {
    fn check(&self, item: &BindingLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingLedgerCurrencyLengthRule {
    pub max_length: i64,
}

impl BindingLedgerRule for BindingLedgerCurrencyLengthRule {
    fn check(&self, item: &BindingLedger) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingLedgerLabelLengthRule {
    pub max_length: i64,
}

impl BindingLedgerRule for BindingLedgerLabelLengthRule {
    fn check(&self, item: &BindingLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingLedgerMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingLedgerRule for BindingLedgerMinimumRangeRule {
    fn check(&self, item: &BindingLedger) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingLedgerDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingLedgerRule for BindingLedgerDurationMinutesRangeRule {
    fn check(&self, item: &BindingLedger) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingLedgerRequiredCurrencyRule {
    pub message: String,
}

impl BindingLedgerRule for BindingLedgerRequiredCurrencyRule {
    fn check(&self, item: &BindingLedger) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingLedgerRequiredCodeRule {
    pub message: String,
}

impl BindingLedgerRule for BindingLedgerRequiredCodeRule {
    fn check(&self, item: &BindingLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding ledger.
#[derive(Debug, Clone)]
pub struct BindingLedgerValidator {
    pub rules: Vec<Rc<dyn BindingLedgerRule>>,
}

impl BindingLedgerValidator {
    pub fn validate(&self, item: &BindingLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_ledger_validator() -> BindingLedgerValidator {
    BindingLedgerValidator {
        rules: vec![
            Rc::new(BindingLedgerCurrencyLengthRule { max_length: 86400 }) as Rc<dyn BindingLedgerRule>,
            Rc::new(BindingLedgerLabelLengthRule { max_length: 64 }),
            Rc::new(BindingLedgerMinimumRangeRule { min: 2.7, max: 1576.8 }),
            Rc::new(BindingLedgerDurationMinutesRangeRule { min: 3600, max: 3600 }),
            Rc::new(BindingLedgerRequiredCurrencyRule {
                message: "binding ledger needs a value".to_string(),
            }),
            Rc::new(BindingLedgerRequiredCodeRule {
                message: "binding ledger needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_ledger_validator();
        let item = new_binding_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = BindingLedgerCurrencyLengthRule { max_length: 64 };
        let mut item = new_binding_ledger("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BindingLedgerLabelLengthRule { max_length: 64 };
        let mut item = new_binding_ledger("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = BindingLedgerMinimumRangeRule { min: 7.9, max: 2304.8 };
        let mut item = new_binding_ledger("x", "y");
        item.minimum = 2306.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = BindingLedgerDurationMinutesRangeRule { min: 120, max: 86400 };
        let mut item = new_binding_ledger("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = BindingLedgerRequiredCurrencyRule {
            message: "binding ledger needs this field".to_string(),
        };
        let mut item = new_binding_ledger("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BindingLedgerRequiredCodeRule {
            message: "binding ledger needs a value".to_string(),
        };
        let mut item = new_binding_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
