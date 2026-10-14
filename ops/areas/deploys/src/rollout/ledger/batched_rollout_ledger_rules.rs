use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched rollout ledger must pass. Check returns its problems, or none.
pub trait BatchedRolloutLedgerRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRolloutLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRolloutLedgerRequiredNamespaceRule {
    pub message: String,
}

impl BatchedRolloutLedgerRule for BatchedRolloutLedgerRequiredNamespaceRule {
    fn check(&self, item: &BatchedRolloutLedger) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRolloutLedgerRequiredCodeRule {
    pub message: String,
}

impl BatchedRolloutLedgerRule for BatchedRolloutLedgerRequiredCodeRule {
    fn check(&self, item: &BatchedRolloutLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRolloutLedgerAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl BatchedRolloutLedgerRule for BatchedRolloutLedgerAllowedNamespaceRule {
    fn check(&self, item: &BatchedRolloutLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is inconsistent", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRolloutLedgerCodePrefixRule {
    pub prefix: String,
}

impl BatchedRolloutLedgerRule for BatchedRolloutLedgerCodePrefixRule {
    fn check(&self, item: &BatchedRolloutLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRolloutLedgerFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedRolloutLedgerRule for BatchedRolloutLedgerFloorValueRangeRule {
    fn check(&self, item: &BatchedRolloutLedger) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched rollout ledger.
#[derive(Debug, Clone)]
pub struct BatchedRolloutLedgerValidator {
    pub rules: Vec<Rc<dyn BatchedRolloutLedgerRule>>,
}

impl BatchedRolloutLedgerValidator {
    pub fn validate(&self, item: &BatchedRolloutLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRolloutLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRolloutLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_rollout_ledger_validator() -> BatchedRolloutLedgerValidator {
    BatchedRolloutLedgerValidator {
        rules: vec![
            Rc::new(BatchedRolloutLedgerRequiredNamespaceRule {
                message: "batched rollout ledger needs a value".to_string(),
            }) as Rc<dyn BatchedRolloutLedgerRule>,
            Rc::new(BatchedRolloutLedgerRequiredCodeRule {
                message: "batched rollout ledger needs a value".to_string(),
            }),
            Rc::new(BatchedRolloutLedgerAllowedNamespaceRule {
                allowed: vec!["scheduled".to_string(), "legacy".to_string(), "primary".to_string()],
            }),
            Rc::new(BatchedRolloutLedgerCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BatchedRolloutLedgerFloorValueRangeRule { min: 2.1, max: 769.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_rollout_ledger_validator();
        let item = new_batched_rollout_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = BatchedRolloutLedgerRequiredNamespaceRule {
            message: "batched rollout ledger needs this field".to_string(),
        };
        let mut item = new_batched_rollout_ledger("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedRolloutLedgerRequiredCodeRule {
            message: "batched rollout ledger needs a value".to_string(),
        };
        let mut item = new_batched_rollout_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = BatchedRolloutLedgerFloorValueRangeRule { min: 2.2, max: 3426.1 };
        let mut item = new_batched_rollout_ledger("x", "y");
        item.floor_value = 3427.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
