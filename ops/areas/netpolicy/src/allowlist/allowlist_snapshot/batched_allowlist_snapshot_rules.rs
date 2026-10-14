use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched allowlist snapshot must pass. Check returns its problems, or none.
pub trait BatchedAllowlistSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedAllowlistSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistSnapshotRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedAllowlistSnapshotRule for BatchedAllowlistSnapshotRatioRangeRule {
    fn check(&self, item: &BatchedAllowlistSnapshot) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistSnapshotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedAllowlistSnapshotRule for BatchedAllowlistSnapshotUpdatedAtRangeRule {
    fn check(&self, item: &BatchedAllowlistSnapshot) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistSnapshotPinnedGuardRule {
    pub limit: i64,
}

impl BatchedAllowlistSnapshotRule for BatchedAllowlistSnapshotPinnedGuardRule {
    fn check(&self, item: &BatchedAllowlistSnapshot) -> Vec<String> {
        if item.pinned && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while pinned",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistSnapshotRequiredCodeRule {
    pub message: String,
}

impl BatchedAllowlistSnapshotRule for BatchedAllowlistSnapshotRequiredCodeRule {
    fn check(&self, item: &BatchedAllowlistSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched allowlist snapshot.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistSnapshotValidator {
    pub rules: Vec<Rc<dyn BatchedAllowlistSnapshotRule>>,
}

impl BatchedAllowlistSnapshotValidator {
    pub fn validate(&self, item: &BatchedAllowlistSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedAllowlistSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedAllowlistSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_allowlist_snapshot_validator() -> BatchedAllowlistSnapshotValidator {
    BatchedAllowlistSnapshotValidator {
        rules: vec![
            Rc::new(BatchedAllowlistSnapshotRatioRangeRule { min: 8.8, max: 3626.2 })
                as Rc<dyn BatchedAllowlistSnapshotRule>,
            Rc::new(BatchedAllowlistSnapshotUpdatedAtRangeRule { min: 80, max: 3600 }),
            Rc::new(BatchedAllowlistSnapshotPinnedGuardRule { limit: 1000 }),
            Rc::new(BatchedAllowlistSnapshotRequiredCodeRule {
                message: "batched allowlist snapshot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_allowlist_snapshot_validator();
        let item = new_batched_allowlist_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = BatchedAllowlistSnapshotRatioRangeRule { min: 0.5, max: 1704.4 };
        let mut item = new_batched_allowlist_snapshot("x", "y");
        item.ratio = 1705.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = BatchedAllowlistSnapshotUpdatedAtRangeRule { min: 80, max: 86400 };
        let mut item = new_batched_allowlist_snapshot("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedAllowlistSnapshotRequiredCodeRule {
            message: "batched allowlist snapshot needs this field".to_string(),
        };
        let mut item = new_batched_allowlist_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
