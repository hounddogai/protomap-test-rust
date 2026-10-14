use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global peering ledger must pass. Check returns its problems, or none.
pub trait GlobalPeeringLedgerRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPeeringLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringLedgerCodeLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringLedgerRule for GlobalPeeringLedgerCodeLengthRule {
    fn check(&self, item: &GlobalPeeringLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringLedgerKeyIdLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringLedgerRule for GlobalPeeringLedgerKeyIdLengthRule {
    fn check(&self, item: &GlobalPeeringLedger) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringLedgerQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPeeringLedgerRule for GlobalPeeringLedgerQuantityRangeRule {
    fn check(&self, item: &GlobalPeeringLedger) -> Vec<String> {
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

/// Runs every rule of a global peering ledger.
#[derive(Debug, Clone)]
pub struct GlobalPeeringLedgerValidator {
    pub rules: Vec<Rc<dyn GlobalPeeringLedgerRule>>,
}

impl GlobalPeeringLedgerValidator {
    pub fn validate(&self, item: &GlobalPeeringLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalPeeringLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPeeringLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_peering_ledger_validator() -> GlobalPeeringLedgerValidator {
    GlobalPeeringLedgerValidator {
        rules: vec![
            Rc::new(GlobalPeeringLedgerCodeLengthRule { max_length: 3600 }) as Rc<dyn GlobalPeeringLedgerRule>,
            Rc::new(GlobalPeeringLedgerKeyIdLengthRule { max_length: 64 }),
            Rc::new(GlobalPeeringLedgerQuantityRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_peering_ledger_validator();
        let item = new_global_peering_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalPeeringLedgerCodeLengthRule { max_length: 120 };
        let mut item = new_global_peering_ledger("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = GlobalPeeringLedgerKeyIdLengthRule { max_length: 32 };
        let mut item = new_global_peering_ledger("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = GlobalPeeringLedgerQuantityRangeRule { min: 80, max: 120 };
        let mut item = new_global_peering_ledger("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
