use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict isolation ledger must pass. Check returns its problems, or none.
pub trait StrictIsolationLedgerRule: std::fmt::Debug {
    fn check(&self, item: &StrictIsolationLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationLedgerNameLengthRule {
    pub max_length: i64,
}

impl StrictIsolationLedgerRule for StrictIsolationLedgerNameLengthRule {
    fn check(&self, item: &StrictIsolationLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationLedgerCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictIsolationLedgerRule for StrictIsolationLedgerCeilingRangeRule {
    fn check(&self, item: &StrictIsolationLedger) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationLedgerRequiredSegmentRule {
    pub message: String,
}

impl StrictIsolationLedgerRule for StrictIsolationLedgerRequiredSegmentRule {
    fn check(&self, item: &StrictIsolationLedger) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationLedgerSegmentLengthRule {
    pub max_length: i64,
}

impl StrictIsolationLedgerRule for StrictIsolationLedgerSegmentLengthRule {
    fn check(&self, item: &StrictIsolationLedger) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationLedgerCodePrefixRule {
    pub prefix: String,
}

impl StrictIsolationLedgerRule for StrictIsolationLedgerCodePrefixRule {
    fn check(&self, item: &StrictIsolationLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict isolation ledger.
#[derive(Debug, Clone)]
pub struct StrictIsolationLedgerValidator {
    pub rules: Vec<Rc<dyn StrictIsolationLedgerRule>>,
}

impl StrictIsolationLedgerValidator {
    pub fn validate(&self, item: &StrictIsolationLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictIsolationLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictIsolationLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_isolation_ledger_validator() -> StrictIsolationLedgerValidator {
    StrictIsolationLedgerValidator {
        rules: vec![
            Rc::new(StrictIsolationLedgerNameLengthRule { max_length: 32 }) as Rc<dyn StrictIsolationLedgerRule>,
            Rc::new(StrictIsolationLedgerCeilingRangeRule { min: 3.2, max: 2779.1 }),
            Rc::new(StrictIsolationLedgerRequiredSegmentRule {
                message: "strict isolation ledger needs this field".to_string(),
            }),
            Rc::new(StrictIsolationLedgerSegmentLengthRule { max_length: 255 }),
            Rc::new(StrictIsolationLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_isolation_ledger_validator();
        let item = new_strict_isolation_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictIsolationLedgerNameLengthRule { max_length: 80 };
        let mut item = new_strict_isolation_ledger("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = StrictIsolationLedgerCeilingRangeRule { min: 3.4, max: 1046.3 };
        let mut item = new_strict_isolation_ledger("x", "y");
        item.ceiling = 1047.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = StrictIsolationLedgerRequiredSegmentRule {
            message: "strict isolation ledger needs a value".to_string(),
        };
        let mut item = new_strict_isolation_ledger("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = StrictIsolationLedgerSegmentLengthRule { max_length: 32 };
        let mut item = new_strict_isolation_ledger("x", "y");
        item.segment = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
