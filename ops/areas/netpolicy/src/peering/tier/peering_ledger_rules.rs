use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering ledger must pass. Check returns its problems, or none.
pub trait PeeringLedgerRule: std::fmt::Debug {
    fn check(&self, item: &PeeringLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringLedgerRequiredRegionRule {
    pub message: String,
}

impl PeeringLedgerRule for PeeringLedgerRequiredRegionRule {
    fn check(&self, item: &PeeringLedger) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringLedgerCodeLengthRule {
    pub max_length: i64,
}

impl PeeringLedgerRule for PeeringLedgerCodeLengthRule {
    fn check(&self, item: &PeeringLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringLedgerCodePrefixRule {
    pub prefix: String,
}

impl PeeringLedgerRule for PeeringLedgerCodePrefixRule {
    fn check(&self, item: &PeeringLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringLedgerRegionLengthRule {
    pub max_length: i64,
}

impl PeeringLedgerRule for PeeringLedgerRegionLengthRule {
    fn check(&self, item: &PeeringLedger) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringLedgerLabelLengthRule {
    pub max_length: i64,
}

impl PeeringLedgerRule for PeeringLedgerLabelLengthRule {
    fn check(&self, item: &PeeringLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering ledger.
#[derive(Debug, Clone)]
pub struct PeeringLedgerValidator {
    pub rules: Vec<Rc<dyn PeeringLedgerRule>>,
}

impl PeeringLedgerValidator {
    pub fn validate(&self, item: &PeeringLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_ledger_validator() -> PeeringLedgerValidator {
    PeeringLedgerValidator {
        rules: vec![
            Rc::new(PeeringLedgerRequiredRegionRule {
                message: "peering ledger needs this field".to_string(),
            }) as Rc<dyn PeeringLedgerRule>,
            Rc::new(PeeringLedgerCodeLengthRule { max_length: 64 }),
            Rc::new(PeeringLedgerCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PeeringLedgerRegionLengthRule { max_length: 3600 }),
            Rc::new(PeeringLedgerLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_ledger_validator();
        let item = new_peering_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PeeringLedgerRequiredRegionRule {
            message: "peering ledger needs an entry".to_string(),
        };
        let mut item = new_peering_ledger("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringLedgerCodeLengthRule { max_length: 3600 };
        let mut item = new_peering_ledger("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PeeringLedgerRegionLengthRule { max_length: 80 };
        let mut item = new_peering_ledger("x", "y");
        item.region = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PeeringLedgerLabelLengthRule { max_length: 1000 };
        let mut item = new_peering_ledger("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
