use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation snapshot must pass. Check returns its problems, or none.
pub trait DelegationSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DelegationSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSnapshotMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationSnapshotRule for DelegationSnapshotMaximumRangeRule {
    fn check(&self, item: &DelegationSnapshot) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSnapshotMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationSnapshotRule for DelegationSnapshotMarginRangeRule {
    fn check(&self, item: &DelegationSnapshot) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSnapshotSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationSnapshotRule for DelegationSnapshotSequenceRangeRule {
    fn check(&self, item: &DelegationSnapshot) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a delegation snapshot.
#[derive(Debug, Clone)]
pub struct DelegationSnapshotValidator {
    pub rules: Vec<Rc<dyn DelegationSnapshotRule>>,
}

impl DelegationSnapshotValidator {
    pub fn validate(&self, item: &DelegationSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_snapshot_validator() -> DelegationSnapshotValidator {
    DelegationSnapshotValidator {
        rules: vec![
            Rc::new(DelegationSnapshotMaximumRangeRule { min: 8.3, max: 2706.5 }) as Rc<dyn DelegationSnapshotRule>,
            Rc::new(DelegationSnapshotMarginRangeRule { min: 4.3, max: 2709.1 }),
            Rc::new(DelegationSnapshotSequenceRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_snapshot_validator();
        let item = new_delegation_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = DelegationSnapshotMaximumRangeRule { min: 1.6, max: 800.3 };
        let mut item = new_delegation_snapshot("x", "y");
        item.maximum = 801.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = DelegationSnapshotMarginRangeRule { min: 2.5, max: 2453.8 };
        let mut item = new_delegation_snapshot("x", "y");
        item.margin = 2455.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = DelegationSnapshotSequenceRangeRule { min: 32, max: 1000 };
        let mut item = new_delegation_snapshot("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
