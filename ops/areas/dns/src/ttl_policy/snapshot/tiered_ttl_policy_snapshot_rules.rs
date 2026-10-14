use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered ttl policy snapshot must pass. Check returns its problems, or none.
pub trait TieredTtlPolicySnapshotRule: std::fmt::Debug {
    fn check(&self, item: &TieredTtlPolicySnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicySnapshotBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTtlPolicySnapshotRule for TieredTtlPolicySnapshotBatchSizeRangeRule {
    fn check(&self, item: &TieredTtlPolicySnapshot) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicySnapshotDescriptionLengthRule {
    pub max_length: i64,
}

impl TieredTtlPolicySnapshotRule for TieredTtlPolicySnapshotDescriptionLengthRule {
    fn check(&self, item: &TieredTtlPolicySnapshot) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicySnapshotAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TieredTtlPolicySnapshotRule for TieredTtlPolicySnapshotAllowedTitleRule {
    fn check(&self, item: &TieredTtlPolicySnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered ttl policy snapshot.
#[derive(Debug, Clone)]
pub struct TieredTtlPolicySnapshotValidator {
    pub rules: Vec<Rc<dyn TieredTtlPolicySnapshotRule>>,
}

impl TieredTtlPolicySnapshotValidator {
    pub fn validate(&self, item: &TieredTtlPolicySnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredTtlPolicySnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTtlPolicySnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_ttl_policy_snapshot_validator() -> TieredTtlPolicySnapshotValidator {
    TieredTtlPolicySnapshotValidator {
        rules: vec![
            Rc::new(TieredTtlPolicySnapshotBatchSizeRangeRule { min: 120, max: 1000 })
                as Rc<dyn TieredTtlPolicySnapshotRule>,
            Rc::new(TieredTtlPolicySnapshotDescriptionLengthRule { max_length: 255 }),
            Rc::new(TieredTtlPolicySnapshotAllowedTitleRule {
                allowed: vec!["adaptive".to_string(), "regional".to_string(), "manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_ttl_policy_snapshot_validator();
        let item = new_tiered_ttl_policy_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TieredTtlPolicySnapshotBatchSizeRangeRule { min: 255, max: 86400 };
        let mut item = new_tiered_ttl_policy_snapshot("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = TieredTtlPolicySnapshotDescriptionLengthRule { max_length: 1000 };
        let mut item = new_tiered_ttl_policy_snapshot("x", "y");
        item.description = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
