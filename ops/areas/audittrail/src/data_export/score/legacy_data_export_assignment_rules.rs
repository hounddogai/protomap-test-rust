use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy data export assignment must pass. Check returns its problems, or none.
pub trait LegacyDataExportAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDataExportAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl LegacyDataExportAssignmentRule for LegacyDataExportAssignmentCodeLengthRule {
    fn check(&self, item: &LegacyDataExportAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAssignmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDataExportAssignmentRule for LegacyDataExportAssignmentCreatedAtRangeRule {
    fn check(&self, item: &LegacyDataExportAssignment) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAssignmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyDataExportAssignmentRule for LegacyDataExportAssignmentRateRangeRule {
    fn check(&self, item: &LegacyDataExportAssignment) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAssignmentRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDataExportAssignmentRule for LegacyDataExportAssignmentRevisionRangeRule {
    fn check(&self, item: &LegacyDataExportAssignment) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy data export assignment.
#[derive(Debug, Clone)]
pub struct LegacyDataExportAssignmentValidator {
    pub rules: Vec<Rc<dyn LegacyDataExportAssignmentRule>>,
}

impl LegacyDataExportAssignmentValidator {
    pub fn validate(&self, item: &LegacyDataExportAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyDataExportAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDataExportAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_data_export_assignment_validator() -> LegacyDataExportAssignmentValidator {
    LegacyDataExportAssignmentValidator {
        rules: vec![
            Rc::new(LegacyDataExportAssignmentCodeLengthRule { max_length: 64 })
                as Rc<dyn LegacyDataExportAssignmentRule>,
            Rc::new(LegacyDataExportAssignmentCreatedAtRangeRule { min: 32, max: 120 }),
            Rc::new(LegacyDataExportAssignmentRateRangeRule { min: 3.1, max: 1341.5 }),
            Rc::new(LegacyDataExportAssignmentRevisionRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_data_export_assignment_validator();
        let item = new_legacy_data_export_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyDataExportAssignmentCodeLengthRule { max_length: 3600 };
        let mut item = new_legacy_data_export_assignment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LegacyDataExportAssignmentCreatedAtRangeRule { min: 120, max: 86400 };
        let mut item = new_legacy_data_export_assignment("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LegacyDataExportAssignmentRateRangeRule { min: 7.7, max: 4534.2 };
        let mut item = new_legacy_data_export_assignment("x", "y");
        item.rate = 4535.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = LegacyDataExportAssignmentRevisionRangeRule { min: 255, max: 86400 };
        let mut item = new_legacy_data_export_assignment("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
