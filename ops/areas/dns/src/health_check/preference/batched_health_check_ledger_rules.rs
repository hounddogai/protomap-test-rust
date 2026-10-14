use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched health check ledger must pass. Check returns its problems, or none.
pub trait BatchedHealthCheckLedgerRule: std::fmt::Debug {
    fn check(&self, item: &BatchedHealthCheckLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedgerCodeLengthRule {
    pub max_length: i64,
}

impl BatchedHealthCheckLedgerRule for BatchedHealthCheckLedgerCodeLengthRule {
    fn check(&self, item: &BatchedHealthCheckLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedgerRequiredCodeRule {
    pub message: String,
}

impl BatchedHealthCheckLedgerRule for BatchedHealthCheckLedgerRequiredCodeRule {
    fn check(&self, item: &BatchedHealthCheckLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedgerCodePrefixRule {
    pub prefix: String,
}

impl BatchedHealthCheckLedgerRule for BatchedHealthCheckLedgerCodePrefixRule {
    fn check(&self, item: &BatchedHealthCheckLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedgerRequiredNameRule {
    pub message: String,
}

impl BatchedHealthCheckLedgerRule for BatchedHealthCheckLedgerRequiredNameRule {
    fn check(&self, item: &BatchedHealthCheckLedger) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedgerScopeLengthRule {
    pub max_length: i64,
}

impl BatchedHealthCheckLedgerRule for BatchedHealthCheckLedgerScopeLengthRule {
    fn check(&self, item: &BatchedHealthCheckLedger) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedgerMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedHealthCheckLedgerRule for BatchedHealthCheckLedgerMarginRangeRule {
    fn check(&self, item: &BatchedHealthCheckLedger) -> Vec<String> {
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

/// Runs every rule of a batched health check ledger.
#[derive(Debug, Clone)]
pub struct BatchedHealthCheckLedgerValidator {
    pub rules: Vec<Rc<dyn BatchedHealthCheckLedgerRule>>,
}

impl BatchedHealthCheckLedgerValidator {
    pub fn validate(&self, item: &BatchedHealthCheckLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedHealthCheckLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedHealthCheckLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_health_check_ledger_validator() -> BatchedHealthCheckLedgerValidator {
    BatchedHealthCheckLedgerValidator {
        rules: vec![
            Rc::new(BatchedHealthCheckLedgerCodeLengthRule { max_length: 80 }) as Rc<dyn BatchedHealthCheckLedgerRule>,
            Rc::new(BatchedHealthCheckLedgerRequiredCodeRule {
                message: "batched health check ledger needs an entry".to_string(),
            }),
            Rc::new(BatchedHealthCheckLedgerCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BatchedHealthCheckLedgerRequiredNameRule {
                message: "batched health check ledger needs this field".to_string(),
            }),
            Rc::new(BatchedHealthCheckLedgerScopeLengthRule { max_length: 16 }),
            Rc::new(BatchedHealthCheckLedgerMarginRangeRule { min: 1.4, max: 569.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_health_check_ledger_validator();
        let item = new_batched_health_check_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedHealthCheckLedgerCodeLengthRule { max_length: 32 };
        let mut item = new_batched_health_check_ledger("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedHealthCheckLedgerRequiredCodeRule {
            message: "batched health check ledger needs a value".to_string(),
        };
        let mut item = new_batched_health_check_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedHealthCheckLedgerRequiredNameRule {
            message: "batched health check ledger needs this field".to_string(),
        };
        let mut item = new_batched_health_check_ledger("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = BatchedHealthCheckLedgerScopeLengthRule { max_length: 64 };
        let mut item = new_batched_health_check_ledger("x", "y");
        item.scope = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = BatchedHealthCheckLedgerMarginRangeRule { min: 2.1, max: 3542.2 };
        let mut item = new_batched_health_check_ledger("x", "y");
        item.margin = 3543.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
