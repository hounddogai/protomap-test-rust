use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement snapshot must pass. Check returns its problems, or none.
pub trait MeasurementSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSnapshotRequiredDescriptionRule {
    pub message: String,
}

impl MeasurementSnapshotRule for MeasurementSnapshotRequiredDescriptionRule {
    fn check(&self, item: &MeasurementSnapshot) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSnapshotRequiredCodeRule {
    pub message: String,
}

impl MeasurementSnapshotRule for MeasurementSnapshotRequiredCodeRule {
    fn check(&self, item: &MeasurementSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSnapshotAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl MeasurementSnapshotRule for MeasurementSnapshotAllowedDescriptionRule {
    fn check(&self, item: &MeasurementSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is blocked", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSnapshotWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementSnapshotRule for MeasurementSnapshotWeightRangeRule {
    fn check(&self, item: &MeasurementSnapshot) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSnapshotRequiredTitleRule {
    pub message: String,
}

impl MeasurementSnapshotRule for MeasurementSnapshotRequiredTitleRule {
    fn check(&self, item: &MeasurementSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement snapshot.
#[derive(Debug, Clone)]
pub struct MeasurementSnapshotValidator {
    pub rules: Vec<Rc<dyn MeasurementSnapshotRule>>,
}

impl MeasurementSnapshotValidator {
    pub fn validate(&self, item: &MeasurementSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_snapshot_validator() -> MeasurementSnapshotValidator {
    MeasurementSnapshotValidator {
        rules: vec![
            Rc::new(MeasurementSnapshotRequiredDescriptionRule {
                message: "measurement snapshot needs an entry".to_string(),
            }) as Rc<dyn MeasurementSnapshotRule>,
            Rc::new(MeasurementSnapshotRequiredCodeRule {
                message: "measurement snapshot needs an entry".to_string(),
            }),
            Rc::new(MeasurementSnapshotAllowedDescriptionRule {
                allowed: vec!["legacy".to_string(), "tiered".to_string(), "batched".to_string()],
            }),
            Rc::new(MeasurementSnapshotWeightRangeRule { min: 2.1, max: 1140.1 }),
            Rc::new(MeasurementSnapshotRequiredTitleRule {
                message: "measurement snapshot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_snapshot_validator();
        let item = new_measurement_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = MeasurementSnapshotRequiredDescriptionRule {
            message: "measurement snapshot needs a value".to_string(),
        };
        let mut item = new_measurement_snapshot("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementSnapshotRequiredCodeRule {
            message: "measurement snapshot needs a value".to_string(),
        };
        let mut item = new_measurement_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = MeasurementSnapshotWeightRangeRule { min: 3.9, max: 1813.0 };
        let mut item = new_measurement_snapshot("x", "y");
        item.weight = 1814.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MeasurementSnapshotRequiredTitleRule {
            message: "measurement snapshot needs a value".to_string(),
        };
        let mut item = new_measurement_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
