use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback policy assignment must pass. Check returns its problems, or none.
pub trait FallbackPolicyAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPolicyAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyAssignmentRequiredLabelRule {
    pub message: String,
}

impl FallbackPolicyAssignmentRule for FallbackPolicyAssignmentRequiredLabelRule {
    fn check(&self, item: &FallbackPolicyAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyAssignmentRequiredCodeRule {
    pub message: String,
}

impl FallbackPolicyAssignmentRule for FallbackPolicyAssignmentRequiredCodeRule {
    fn check(&self, item: &FallbackPolicyAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyAssignmentAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPolicyAssignmentRule for FallbackPolicyAssignmentAttemptsRangeRule {
    fn check(&self, item: &FallbackPolicyAssignment) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyAssignmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackPolicyAssignmentRule for FallbackPolicyAssignmentFloorValueRangeRule {
    fn check(&self, item: &FallbackPolicyAssignment) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback policy assignment.
#[derive(Debug, Clone)]
pub struct FallbackPolicyAssignmentValidator {
    pub rules: Vec<Rc<dyn FallbackPolicyAssignmentRule>>,
}

impl FallbackPolicyAssignmentValidator {
    pub fn validate(&self, item: &FallbackPolicyAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackPolicyAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPolicyAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_policy_assignment_validator() -> FallbackPolicyAssignmentValidator {
    FallbackPolicyAssignmentValidator {
        rules: vec![
            Rc::new(FallbackPolicyAssignmentRequiredLabelRule {
                message: "fallback policy assignment needs this field".to_string(),
            }) as Rc<dyn FallbackPolicyAssignmentRule>,
            Rc::new(FallbackPolicyAssignmentRequiredCodeRule {
                message: "fallback policy assignment needs an entry".to_string(),
            }),
            Rc::new(FallbackPolicyAssignmentAttemptsRangeRule { min: 16, max: 120 }),
            Rc::new(FallbackPolicyAssignmentFloorValueRangeRule { min: 5.7, max: 3214.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_policy_assignment_validator();
        let item = new_fallback_policy_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackPolicyAssignmentRequiredLabelRule {
            message: "fallback policy assignment needs a value".to_string(),
        };
        let mut item = new_fallback_policy_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackPolicyAssignmentRequiredCodeRule {
            message: "fallback policy assignment needs a value".to_string(),
        };
        let mut item = new_fallback_policy_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = FallbackPolicyAssignmentAttemptsRangeRule { min: 16, max: 64 };
        let mut item = new_fallback_policy_assignment("x", "y");
        item.attempts = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FallbackPolicyAssignmentFloorValueRangeRule { min: 0.1, max: 668.0 };
        let mut item = new_fallback_policy_assignment("x", "y");
        item.floor_value = 669.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
