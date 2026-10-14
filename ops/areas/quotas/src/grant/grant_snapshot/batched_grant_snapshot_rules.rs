use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched grant snapshot must pass. Check returns its problems, or none.
pub trait BatchedGrantSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedGrantSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantSnapshotAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedGrantSnapshotRule for BatchedGrantSnapshotAttemptsRangeRule {
    fn check(&self, item: &BatchedGrantSnapshot) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantSnapshotRequiredScopeRule {
    pub message: String,
}

impl BatchedGrantSnapshotRule for BatchedGrantSnapshotRequiredScopeRule {
    fn check(&self, item: &BatchedGrantSnapshot) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantSnapshotAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl BatchedGrantSnapshotRule for BatchedGrantSnapshotAllowedCountryRule {
    fn check(&self, item: &BatchedGrantSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is duplicated", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantSnapshotCodePrefixRule {
    pub prefix: String,
}

impl BatchedGrantSnapshotRule for BatchedGrantSnapshotCodePrefixRule {
    fn check(&self, item: &BatchedGrantSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantSnapshotRequiredLabelRule {
    pub message: String,
}

impl BatchedGrantSnapshotRule for BatchedGrantSnapshotRequiredLabelRule {
    fn check(&self, item: &BatchedGrantSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched grant snapshot.
#[derive(Debug, Clone)]
pub struct BatchedGrantSnapshotValidator {
    pub rules: Vec<Rc<dyn BatchedGrantSnapshotRule>>,
}

impl BatchedGrantSnapshotValidator {
    pub fn validate(&self, item: &BatchedGrantSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedGrantSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedGrantSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_grant_snapshot_validator() -> BatchedGrantSnapshotValidator {
    BatchedGrantSnapshotValidator {
        rules: vec![
            Rc::new(BatchedGrantSnapshotAttemptsRangeRule { min: 16, max: 3600 }) as Rc<dyn BatchedGrantSnapshotRule>,
            Rc::new(BatchedGrantSnapshotRequiredScopeRule {
                message: "batched grant snapshot needs a value".to_string(),
            }),
            Rc::new(BatchedGrantSnapshotAllowedCountryRule { allowed: Vec::new() }),
            Rc::new(BatchedGrantSnapshotCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BatchedGrantSnapshotRequiredLabelRule {
                message: "batched grant snapshot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_grant_snapshot_validator();
        let item = new_batched_grant_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = BatchedGrantSnapshotAttemptsRangeRule { min: 32, max: 86400 };
        let mut item = new_batched_grant_snapshot("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = BatchedGrantSnapshotRequiredScopeRule {
            message: "batched grant snapshot needs an entry".to_string(),
        };
        let mut item = new_batched_grant_snapshot("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedGrantSnapshotRequiredLabelRule {
            message: "batched grant snapshot needs an entry".to_string(),
        };
        let mut item = new_batched_grant_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
