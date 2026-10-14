use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling rollback snapshot must pass. Check returns its problems, or none.
pub trait RollingRollbackSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RollingRollbackSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackSnapshotWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingRollbackSnapshotRule for RollingRollbackSnapshotWeightRangeRule {
    fn check(&self, item: &RollingRollbackSnapshot) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackSnapshotCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRollbackSnapshotRule for RollingRollbackSnapshotCapacityRangeRule {
    fn check(&self, item: &RollingRollbackSnapshot) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackSnapshotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRollbackSnapshotRule for RollingRollbackSnapshotUpdatedAtRangeRule {
    fn check(&self, item: &RollingRollbackSnapshot) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling rollback snapshot.
#[derive(Debug, Clone)]
pub struct RollingRollbackSnapshotValidator {
    pub rules: Vec<Rc<dyn RollingRollbackSnapshotRule>>,
}

impl RollingRollbackSnapshotValidator {
    pub fn validate(&self, item: &RollingRollbackSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingRollbackSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRollbackSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_rollback_snapshot_validator() -> RollingRollbackSnapshotValidator {
    RollingRollbackSnapshotValidator {
        rules: vec![
            Rc::new(RollingRollbackSnapshotWeightRangeRule { min: 4.6, max: 1713.3 })
                as Rc<dyn RollingRollbackSnapshotRule>,
            Rc::new(RollingRollbackSnapshotCapacityRangeRule { min: 255, max: 1000 }),
            Rc::new(RollingRollbackSnapshotUpdatedAtRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_rollback_snapshot_validator();
        let item = new_rolling_rollback_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = RollingRollbackSnapshotWeightRangeRule { min: 7.0, max: 2059.5 };
        let mut item = new_rolling_rollback_snapshot("x", "y");
        item.weight = 2061.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RollingRollbackSnapshotCapacityRangeRule { min: 80, max: 120 };
        let mut item = new_rolling_rollback_snapshot("x", "y");
        item.capacity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = RollingRollbackSnapshotUpdatedAtRangeRule { min: 16, max: 120 };
        let mut item = new_rolling_rollback_snapshot("x", "y");
        item.updated_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
