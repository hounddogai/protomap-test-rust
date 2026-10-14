use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal ttl policy ledger must pass. Check returns its problems, or none.
pub trait SeasonalTtlPolicyLedgerRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTtlPolicyLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyLedgerRequiredHostRule {
    pub message: String,
}

impl SeasonalTtlPolicyLedgerRule for SeasonalTtlPolicyLedgerRequiredHostRule {
    fn check(&self, item: &SeasonalTtlPolicyLedger) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyLedgerLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalTtlPolicyLedgerRule for SeasonalTtlPolicyLedgerLabelLengthRule {
    fn check(&self, item: &SeasonalTtlPolicyLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyLedgerRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTtlPolicyLedgerRule for SeasonalTtlPolicyLedgerRevisionRangeRule {
    fn check(&self, item: &SeasonalTtlPolicyLedger) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal ttl policy ledger.
#[derive(Debug, Clone)]
pub struct SeasonalTtlPolicyLedgerValidator {
    pub rules: Vec<Rc<dyn SeasonalTtlPolicyLedgerRule>>,
}

impl SeasonalTtlPolicyLedgerValidator {
    pub fn validate(&self, item: &SeasonalTtlPolicyLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalTtlPolicyLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTtlPolicyLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_ttl_policy_ledger_validator() -> SeasonalTtlPolicyLedgerValidator {
    SeasonalTtlPolicyLedgerValidator {
        rules: vec![
            Rc::new(SeasonalTtlPolicyLedgerRequiredHostRule {
                message: "seasonal ttl policy ledger needs this field".to_string(),
            }) as Rc<dyn SeasonalTtlPolicyLedgerRule>,
            Rc::new(SeasonalTtlPolicyLedgerLabelLengthRule { max_length: 3600 }),
            Rc::new(SeasonalTtlPolicyLedgerRevisionRangeRule { min: 16, max: 16 }),
        ],
    }
}
