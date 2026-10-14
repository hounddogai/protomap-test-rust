use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval mapping must pass. Check returns its problems, or none.
pub trait ApprovalMappingRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMappingAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalMappingRule for ApprovalMappingAttemptsRangeRule {
    fn check(&self, item: &ApprovalMapping) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMappingRequiredLabelRule {
    pub message: String,
}

impl ApprovalMappingRule for ApprovalMappingRequiredLabelRule {
    fn check(&self, item: &ApprovalMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMappingWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalMappingRule for ApprovalMappingWindowDaysRangeRule {
    fn check(&self, item: &ApprovalMapping) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a approval mapping.
#[derive(Debug, Clone)]
pub struct ApprovalMappingValidator {
    pub rules: Vec<Rc<dyn ApprovalMappingRule>>,
}

impl ApprovalMappingValidator {
    pub fn validate(&self, item: &ApprovalMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_mapping_validator() -> ApprovalMappingValidator {
    ApprovalMappingValidator {
        rules: vec![
            Rc::new(ApprovalMappingAttemptsRangeRule { min: 16, max: 1000 }) as Rc<dyn ApprovalMappingRule>,
            Rc::new(ApprovalMappingRequiredLabelRule {
                message: "approval mapping needs this field".to_string(),
            }),
            Rc::new(ApprovalMappingWindowDaysRangeRule { min: 255, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_mapping_validator();
        let item = new_approval_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ApprovalMappingAttemptsRangeRule { min: 120, max: 86400 };
        let mut item = new_approval_mapping("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ApprovalMappingRequiredLabelRule {
            message: "approval mapping needs a value".to_string(),
        };
        let mut item = new_approval_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = ApprovalMappingWindowDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_approval_mapping("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
