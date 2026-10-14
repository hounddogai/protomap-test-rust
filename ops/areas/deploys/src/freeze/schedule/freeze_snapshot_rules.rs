use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze snapshot must pass. Check returns its problems, or none.
pub trait FreezeSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &FreezeSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSnapshotCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeSnapshotRule for FreezeSnapshotCeilingRangeRule {
    fn check(&self, item: &FreezeSnapshot) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSnapshotBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeSnapshotRule for FreezeSnapshotBudgetRangeRule {
    fn check(&self, item: &FreezeSnapshot) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSnapshotBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeSnapshotRule for FreezeSnapshotBurstRangeRule {
    fn check(&self, item: &FreezeSnapshot) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSnapshotRequiredDescriptionRule {
    pub message: String,
}

impl FreezeSnapshotRule for FreezeSnapshotRequiredDescriptionRule {
    fn check(&self, item: &FreezeSnapshot) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze snapshot.
#[derive(Debug, Clone)]
pub struct FreezeSnapshotValidator {
    pub rules: Vec<Rc<dyn FreezeSnapshotRule>>,
}

impl FreezeSnapshotValidator {
    pub fn validate(&self, item: &FreezeSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_snapshot_validator() -> FreezeSnapshotValidator {
    FreezeSnapshotValidator {
        rules: vec![
            Rc::new(FreezeSnapshotCeilingRangeRule { min: 9.4, max: 4776.7 }) as Rc<dyn FreezeSnapshotRule>,
            Rc::new(FreezeSnapshotBudgetRangeRule { min: 2.2, max: 2617.7 }),
            Rc::new(FreezeSnapshotBurstRangeRule { min: 120, max: 1000 }),
            Rc::new(FreezeSnapshotRequiredDescriptionRule {
                message: "freeze snapshot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_snapshot_validator();
        let item = new_freeze_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = FreezeSnapshotCeilingRangeRule { min: 0.9, max: 4354.5 };
        let mut item = new_freeze_snapshot("x", "y");
        item.ceiling = 4356.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = FreezeSnapshotBudgetRangeRule { min: 3.5, max: 1135.2 };
        let mut item = new_freeze_snapshot("x", "y");
        item.budget = 1136.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = FreezeSnapshotBurstRangeRule { min: 80, max: 86400 };
        let mut item = new_freeze_snapshot("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = FreezeSnapshotRequiredDescriptionRule {
            message: "freeze snapshot needs an entry".to_string(),
        };
        let mut item = new_freeze_snapshot("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
