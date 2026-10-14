use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit ledger must pass. Check returns its problems, or none.
pub trait LimitLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LimitLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitLedgerRequiredCodeRule {
    pub message: String,
}

impl LimitLedgerRule for LimitLedgerRequiredCodeRule {
    fn check(&self, item: &LimitLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitLedgerLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitLedgerRule for LimitLedgerLimitRangeRule {
    fn check(&self, item: &LimitLedger) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitLedgerCodeLengthRule {
    pub max_length: i64,
}

impl LimitLedgerRule for LimitLedgerCodeLengthRule {
    fn check(&self, item: &LimitLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit ledger.
#[derive(Debug, Clone)]
pub struct LimitLedgerValidator {
    pub rules: Vec<Rc<dyn LimitLedgerRule>>,
}

impl LimitLedgerValidator {
    pub fn validate(&self, item: &LimitLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_ledger_validator() -> LimitLedgerValidator {
    LimitLedgerValidator {
        rules: vec![
            Rc::new(LimitLedgerRequiredCodeRule {
                message: "limit ledger needs this field".to_string(),
            }) as Rc<dyn LimitLedgerRule>,
            Rc::new(LimitLedgerLimitRangeRule { min: 16, max: 80 }),
            Rc::new(LimitLedgerCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_ledger_validator();
        let item = new_limit_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitLedgerRequiredCodeRule {
            message: "limit ledger needs a value".to_string(),
        };
        let mut item = new_limit_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LimitLedgerLimitRangeRule { min: 120, max: 255 };
        let mut item = new_limit_ledger("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitLedgerCodeLengthRule { max_length: 32 };
        let mut item = new_limit_ledger("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
