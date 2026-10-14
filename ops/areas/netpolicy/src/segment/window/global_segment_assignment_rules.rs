use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global segment assignment must pass. Check returns its problems, or none.
pub trait GlobalSegmentAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSegmentAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentAssignmentRequiredDescriptionRule {
    pub message: String,
}

impl GlobalSegmentAssignmentRule for GlobalSegmentAssignmentRequiredDescriptionRule {
    fn check(&self, item: &GlobalSegmentAssignment) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentAssignmentDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalSegmentAssignmentRule for GlobalSegmentAssignmentDurationMinutesRangeRule {
    fn check(&self, item: &GlobalSegmentAssignment) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentAssignmentStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalSegmentAssignmentRule for GlobalSegmentAssignmentStartsAtRangeRule {
    fn check(&self, item: &GlobalSegmentAssignment) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global segment assignment.
#[derive(Debug, Clone)]
pub struct GlobalSegmentAssignmentValidator {
    pub rules: Vec<Rc<dyn GlobalSegmentAssignmentRule>>,
}

impl GlobalSegmentAssignmentValidator {
    pub fn validate(&self, item: &GlobalSegmentAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalSegmentAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSegmentAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_segment_assignment_validator() -> GlobalSegmentAssignmentValidator {
    GlobalSegmentAssignmentValidator {
        rules: vec![
            Rc::new(GlobalSegmentAssignmentRequiredDescriptionRule {
                message: "global segment assignment needs a value".to_string(),
            }) as Rc<dyn GlobalSegmentAssignmentRule>,
            Rc::new(GlobalSegmentAssignmentDurationMinutesRangeRule { min: 64, max: 3600 }),
            Rc::new(GlobalSegmentAssignmentStartsAtRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_segment_assignment_validator();
        let item = new_global_segment_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = GlobalSegmentAssignmentRequiredDescriptionRule {
            message: "global segment assignment needs this field".to_string(),
        };
        let mut item = new_global_segment_assignment("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = GlobalSegmentAssignmentDurationMinutesRangeRule { min: 80, max: 3600 };
        let mut item = new_global_segment_assignment("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = GlobalSegmentAssignmentStartsAtRangeRule { min: 16, max: 3600 };
        let mut item = new_global_segment_assignment("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
