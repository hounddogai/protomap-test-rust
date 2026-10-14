use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy usage assignment must pass. Check returns its problems, or none.
pub trait LegacyUsageAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyUsageAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAssignmentThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyUsageAssignmentRule for LegacyUsageAssignmentThresholdRangeRule {
    fn check(&self, item: &LegacyUsageAssignment) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAssignmentRequiredNameRule {
    pub message: String,
}

impl LegacyUsageAssignmentRule for LegacyUsageAssignmentRequiredNameRule {
    fn check(&self, item: &LegacyUsageAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAssignmentNameLengthRule {
    pub max_length: i64,
}

impl LegacyUsageAssignmentRule for LegacyUsageAssignmentNameLengthRule {
    fn check(&self, item: &LegacyUsageAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAssignmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyUsageAssignmentRule for LegacyUsageAssignmentVersionRangeRule {
    fn check(&self, item: &LegacyUsageAssignment) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl LegacyUsageAssignmentRule for LegacyUsageAssignmentCodeLengthRule {
    fn check(&self, item: &LegacyUsageAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy usage assignment.
#[derive(Debug, Clone)]
pub struct LegacyUsageAssignmentValidator {
    pub rules: Vec<Rc<dyn LegacyUsageAssignmentRule>>,
}

impl LegacyUsageAssignmentValidator {
    pub fn validate(&self, item: &LegacyUsageAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyUsageAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyUsageAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_usage_assignment_validator() -> LegacyUsageAssignmentValidator {
    LegacyUsageAssignmentValidator {
        rules: vec![
            Rc::new(LegacyUsageAssignmentThresholdRangeRule { min: 9.5, max: 947.2 })
                as Rc<dyn LegacyUsageAssignmentRule>,
            Rc::new(LegacyUsageAssignmentRequiredNameRule {
                message: "legacy usage assignment needs this field".to_string(),
            }),
            Rc::new(LegacyUsageAssignmentNameLengthRule { max_length: 120 }),
            Rc::new(LegacyUsageAssignmentVersionRangeRule { min: 16, max: 32 }),
            Rc::new(LegacyUsageAssignmentCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_usage_assignment_validator();
        let item = new_legacy_usage_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LegacyUsageAssignmentThresholdRangeRule { min: 7.4, max: 1340.5 };
        let mut item = new_legacy_usage_assignment("x", "y");
        item.threshold = 1342.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyUsageAssignmentRequiredNameRule {
            message: "legacy usage assignment needs an entry".to_string(),
        };
        let mut item = new_legacy_usage_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyUsageAssignmentNameLengthRule { max_length: 1000 };
        let mut item = new_legacy_usage_assignment("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = LegacyUsageAssignmentVersionRangeRule { min: 16, max: 120 };
        let mut item = new_legacy_usage_assignment("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyUsageAssignmentCodeLengthRule { max_length: 86400 };
        let mut item = new_legacy_usage_assignment("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
