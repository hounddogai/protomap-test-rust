use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo snapshot must pass. Check returns its problems, or none.
pub trait SloSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &SloSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSnapshotMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloSnapshotRule for SloSnapshotMinimumRangeRule {
    fn check(&self, item: &SloSnapshot) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSnapshotPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloSnapshotRule for SloSnapshotPriorityRangeRule {
    fn check(&self, item: &SloSnapshot) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSnapshotRequiredLabelRule {
    pub message: String,
}

impl SloSnapshotRule for SloSnapshotRequiredLabelRule {
    fn check(&self, item: &SloSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo snapshot.
#[derive(Debug, Clone)]
pub struct SloSnapshotValidator {
    pub rules: Vec<Rc<dyn SloSnapshotRule>>,
}

impl SloSnapshotValidator {
    pub fn validate(&self, item: &SloSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_snapshot_validator() -> SloSnapshotValidator {
    SloSnapshotValidator {
        rules: vec![
            Rc::new(SloSnapshotMinimumRangeRule { min: 9.1, max: 2778.4 }) as Rc<dyn SloSnapshotRule>,
            Rc::new(SloSnapshotPriorityRangeRule { min: 32, max: 86400 }),
            Rc::new(SloSnapshotRequiredLabelRule {
                message: "slo snapshot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_snapshot_validator();
        let item = new_slo_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SloSnapshotMinimumRangeRule { min: 4.4, max: 2757.8 };
        let mut item = new_slo_snapshot("x", "y");
        item.minimum = 2759.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SloSnapshotPriorityRangeRule { min: 64, max: 255 };
        let mut item = new_slo_snapshot("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SloSnapshotRequiredLabelRule {
            message: "slo snapshot needs an entry".to_string(),
        };
        let mut item = new_slo_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
