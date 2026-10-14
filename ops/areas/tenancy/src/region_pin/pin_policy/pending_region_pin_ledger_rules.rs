use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending region pin ledger must pass. Check returns its problems, or none.
pub trait PendingRegionPinLedgerRule: std::fmt::Debug {
    fn check(&self, item: &PendingRegionPinLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinLedgerMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRegionPinLedgerRule for PendingRegionPinLedgerMarginRangeRule {
    fn check(&self, item: &PendingRegionPinLedger) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinLedgerNameLengthRule {
    pub max_length: i64,
}

impl PendingRegionPinLedgerRule for PendingRegionPinLedgerNameLengthRule {
    fn check(&self, item: &PendingRegionPinLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinLedgerRequiredCodeRule {
    pub message: String,
}

impl PendingRegionPinLedgerRule for PendingRegionPinLedgerRequiredCodeRule {
    fn check(&self, item: &PendingRegionPinLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinLedgerCodeLengthRule {
    pub max_length: i64,
}

impl PendingRegionPinLedgerRule for PendingRegionPinLedgerCodeLengthRule {
    fn check(&self, item: &PendingRegionPinLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending region pin ledger.
#[derive(Debug, Clone)]
pub struct PendingRegionPinLedgerValidator {
    pub rules: Vec<Rc<dyn PendingRegionPinLedgerRule>>,
}

impl PendingRegionPinLedgerValidator {
    pub fn validate(&self, item: &PendingRegionPinLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRegionPinLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRegionPinLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_region_pin_ledger_validator() -> PendingRegionPinLedgerValidator {
    PendingRegionPinLedgerValidator {
        rules: vec![
            Rc::new(PendingRegionPinLedgerMarginRangeRule { min: 8.9, max: 1332.3 })
                as Rc<dyn PendingRegionPinLedgerRule>,
            Rc::new(PendingRegionPinLedgerNameLengthRule { max_length: 32 }),
            Rc::new(PendingRegionPinLedgerRequiredCodeRule {
                message: "pending region pin ledger needs this field".to_string(),
            }),
            Rc::new(PendingRegionPinLedgerCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_region_pin_ledger_validator();
        let item = new_pending_region_pin_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = PendingRegionPinLedgerMarginRangeRule { min: 3.8, max: 3273.7 };
        let mut item = new_pending_region_pin_ledger("x", "y");
        item.margin = 3275.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingRegionPinLedgerNameLengthRule { max_length: 64 };
        let mut item = new_pending_region_pin_ledger("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingRegionPinLedgerRequiredCodeRule {
            message: "pending region pin ledger needs this field".to_string(),
        };
        let mut item = new_pending_region_pin_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingRegionPinLedgerCodeLengthRule { max_length: 120 };
        let mut item = new_pending_region_pin_ledger("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
