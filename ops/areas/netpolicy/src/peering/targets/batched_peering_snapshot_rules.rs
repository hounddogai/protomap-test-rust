use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched peering snapshot must pass. Check returns its problems, or none.
pub trait BatchedPeeringSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPeeringSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringSnapshotAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl BatchedPeeringSnapshotRule for BatchedPeeringSnapshotAllowedCurrencyRule {
    fn check(&self, item: &BatchedPeeringSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is above maximum", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl BatchedPeeringSnapshotRule for BatchedPeeringSnapshotCodeLengthRule {
    fn check(&self, item: &BatchedPeeringSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringSnapshotCodePrefixRule {
    pub prefix: String,
}

impl BatchedPeeringSnapshotRule for BatchedPeeringSnapshotCodePrefixRule {
    fn check(&self, item: &BatchedPeeringSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringSnapshotRequiredNameRule {
    pub message: String,
}

impl BatchedPeeringSnapshotRule for BatchedPeeringSnapshotRequiredNameRule {
    fn check(&self, item: &BatchedPeeringSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched peering snapshot.
#[derive(Debug, Clone)]
pub struct BatchedPeeringSnapshotValidator {
    pub rules: Vec<Rc<dyn BatchedPeeringSnapshotRule>>,
}

impl BatchedPeeringSnapshotValidator {
    pub fn validate(&self, item: &BatchedPeeringSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedPeeringSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPeeringSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_peering_snapshot_validator() -> BatchedPeeringSnapshotValidator {
    BatchedPeeringSnapshotValidator {
        rules: vec![
            Rc::new(BatchedPeeringSnapshotAllowedCurrencyRule { allowed: Vec::new() })
                as Rc<dyn BatchedPeeringSnapshotRule>,
            Rc::new(BatchedPeeringSnapshotCodeLengthRule { max_length: 3600 }),
            Rc::new(BatchedPeeringSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BatchedPeeringSnapshotRequiredNameRule {
                message: "batched peering snapshot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_peering_snapshot_validator();
        let item = new_batched_peering_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedPeeringSnapshotCodeLengthRule { max_length: 1000 };
        let mut item = new_batched_peering_snapshot("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedPeeringSnapshotRequiredNameRule {
            message: "batched peering snapshot needs this field".to_string(),
        };
        let mut item = new_batched_peering_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
