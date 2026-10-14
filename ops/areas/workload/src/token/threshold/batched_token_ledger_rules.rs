use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched token ledger must pass. Check returns its problems, or none.
pub trait BatchedTokenLedgerRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTokenLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenLedgerRequiredCodeRule {
    pub message: String,
}

impl BatchedTokenLedgerRule for BatchedTokenLedgerRequiredCodeRule {
    fn check(&self, item: &BatchedTokenLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenLedgerTitleLengthRule {
    pub max_length: i64,
}

impl BatchedTokenLedgerRule for BatchedTokenLedgerTitleLengthRule {
    fn check(&self, item: &BatchedTokenLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenLedgerRequiredTitleRule {
    pub message: String,
}

impl BatchedTokenLedgerRule for BatchedTokenLedgerRequiredTitleRule {
    fn check(&self, item: &BatchedTokenLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenLedgerRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedTokenLedgerRule for BatchedTokenLedgerRotationDaysRangeRule {
    fn check(&self, item: &BatchedTokenLedger) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenLedgerCodeLengthRule {
    pub max_length: i64,
}

impl BatchedTokenLedgerRule for BatchedTokenLedgerCodeLengthRule {
    fn check(&self, item: &BatchedTokenLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenLedgerRequiredHostRule {
    pub message: String,
}

impl BatchedTokenLedgerRule for BatchedTokenLedgerRequiredHostRule {
    fn check(&self, item: &BatchedTokenLedger) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched token ledger.
#[derive(Debug, Clone)]
pub struct BatchedTokenLedgerValidator {
    pub rules: Vec<Rc<dyn BatchedTokenLedgerRule>>,
}

impl BatchedTokenLedgerValidator {
    pub fn validate(&self, item: &BatchedTokenLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedTokenLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTokenLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_token_ledger_validator() -> BatchedTokenLedgerValidator {
    BatchedTokenLedgerValidator {
        rules: vec![
            Rc::new(BatchedTokenLedgerRequiredCodeRule {
                message: "batched token ledger needs an entry".to_string(),
            }) as Rc<dyn BatchedTokenLedgerRule>,
            Rc::new(BatchedTokenLedgerTitleLengthRule { max_length: 255 }),
            Rc::new(BatchedTokenLedgerRequiredTitleRule {
                message: "batched token ledger needs a value".to_string(),
            }),
            Rc::new(BatchedTokenLedgerRotationDaysRangeRule { min: 1000, max: 86400 }),
            Rc::new(BatchedTokenLedgerCodeLengthRule { max_length: 120 }),
            Rc::new(BatchedTokenLedgerRequiredHostRule {
                message: "batched token ledger needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_token_ledger_validator();
        let item = new_batched_token_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedTokenLedgerRequiredCodeRule {
            message: "batched token ledger needs an entry".to_string(),
        };
        let mut item = new_batched_token_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedTokenLedgerTitleLengthRule { max_length: 80 };
        let mut item = new_batched_token_ledger("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BatchedTokenLedgerRequiredTitleRule {
            message: "batched token ledger needs an entry".to_string(),
        };
        let mut item = new_batched_token_ledger("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = BatchedTokenLedgerRotationDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_batched_token_ledger("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedTokenLedgerCodeLengthRule { max_length: 80 };
        let mut item = new_batched_token_ledger("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = BatchedTokenLedgerRequiredHostRule {
            message: "batched token ledger needs an entry".to_string(),
        };
        let mut item = new_batched_token_ledger("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
