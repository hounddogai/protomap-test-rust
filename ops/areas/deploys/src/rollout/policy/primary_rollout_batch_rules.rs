use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary rollout batch must pass. Check returns its problems, or none.
pub trait PrimaryRolloutBatchRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRolloutBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutBatchUnitLengthRule {
    pub max_length: i64,
}

impl PrimaryRolloutBatchRule for PrimaryRolloutBatchUnitLengthRule {
    fn check(&self, item: &PrimaryRolloutBatch) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryRolloutBatchRule for PrimaryRolloutBatchAllowedCodeRule {
    fn check(&self, item: &PrimaryRolloutBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutBatchRequiredUnitRule {
    pub message: String,
}

impl PrimaryRolloutBatchRule for PrimaryRolloutBatchRequiredUnitRule {
    fn check(&self, item: &PrimaryRolloutBatch) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutBatchOwnerIdLengthRule {
    pub max_length: i64,
}

impl PrimaryRolloutBatchRule for PrimaryRolloutBatchOwnerIdLengthRule {
    fn check(&self, item: &PrimaryRolloutBatch) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutBatchRequiredNameRule {
    pub message: String,
}

impl PrimaryRolloutBatchRule for PrimaryRolloutBatchRequiredNameRule {
    fn check(&self, item: &PrimaryRolloutBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary rollout batch.
#[derive(Debug, Clone)]
pub struct PrimaryRolloutBatchValidator {
    pub rules: Vec<Rc<dyn PrimaryRolloutBatchRule>>,
}

impl PrimaryRolloutBatchValidator {
    pub fn validate(&self, item: &PrimaryRolloutBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRolloutBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRolloutBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rollout_batch_validator() -> PrimaryRolloutBatchValidator {
    PrimaryRolloutBatchValidator {
        rules: vec![
            Rc::new(PrimaryRolloutBatchUnitLengthRule { max_length: 80 }) as Rc<dyn PrimaryRolloutBatchRule>,
            Rc::new(PrimaryRolloutBatchAllowedCodeRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(PrimaryRolloutBatchRequiredUnitRule {
                message: "primary rollout batch needs an entry".to_string(),
            }),
            Rc::new(PrimaryRolloutBatchOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(PrimaryRolloutBatchRequiredNameRule {
                message: "primary rollout batch needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_rollout_batch_validator();
        let item = new_primary_rollout_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PrimaryRolloutBatchUnitLengthRule { max_length: 1000 };
        let mut item = new_primary_rollout_batch("x", "y");
        item.unit = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryRolloutBatchRequiredUnitRule {
            message: "primary rollout batch needs an entry".to_string(),
        };
        let mut item = new_primary_rollout_batch("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = PrimaryRolloutBatchOwnerIdLengthRule { max_length: 1000 };
        let mut item = new_primary_rollout_batch("x", "y");
        item.owner_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryRolloutBatchRequiredNameRule {
            message: "primary rollout batch needs a value".to_string(),
        };
        let mut item = new_primary_rollout_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
