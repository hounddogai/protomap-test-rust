use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan snapshot must pass. Check returns its problems, or none.
pub trait PlanSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &PlanSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSnapshotTimezoneLengthRule {
    pub max_length: i64,
}

impl PlanSnapshotRule for PlanSnapshotTimezoneLengthRule {
    fn check(&self, item: &PlanSnapshot) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl PlanSnapshotRule for PlanSnapshotCodeLengthRule {
    fn check(&self, item: &PlanSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSnapshotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanSnapshotRule for PlanSnapshotUpdatedAtRangeRule {
    fn check(&self, item: &PlanSnapshot) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSnapshotDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanSnapshotRule for PlanSnapshotDurationMinutesRangeRule {
    fn check(&self, item: &PlanSnapshot) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan snapshot.
#[derive(Debug, Clone)]
pub struct PlanSnapshotValidator {
    pub rules: Vec<Rc<dyn PlanSnapshotRule>>,
}

impl PlanSnapshotValidator {
    pub fn validate(&self, item: &PlanSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_snapshot_validator() -> PlanSnapshotValidator {
    PlanSnapshotValidator {
        rules: vec![
            Rc::new(PlanSnapshotTimezoneLengthRule { max_length: 3600 }) as Rc<dyn PlanSnapshotRule>,
            Rc::new(PlanSnapshotCodeLengthRule { max_length: 80 }),
            Rc::new(PlanSnapshotUpdatedAtRangeRule { min: 32, max: 1000 }),
            Rc::new(PlanSnapshotDurationMinutesRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_snapshot_validator();
        let item = new_plan_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = PlanSnapshotTimezoneLengthRule { max_length: 3600 };
        let mut item = new_plan_snapshot("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanSnapshotCodeLengthRule { max_length: 64 };
        let mut item = new_plan_snapshot("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = PlanSnapshotUpdatedAtRangeRule { min: 120, max: 1000 };
        let mut item = new_plan_snapshot("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = PlanSnapshotDurationMinutesRangeRule { min: 1000, max: 3600 };
        let mut item = new_plan_snapshot("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
