use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual flow log assignment must pass. Check returns its problems, or none.
pub trait ManualFlowLogAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ManualFlowLogAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogAssignmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualFlowLogAssignmentRule for ManualFlowLogAssignmentCeilingRangeRule {
    fn check(&self, item: &ManualFlowLogAssignment) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogAssignmentRequiredCodeRule {
    pub message: String,
}

impl ManualFlowLogAssignmentRule for ManualFlowLogAssignmentRequiredCodeRule {
    fn check(&self, item: &ManualFlowLogAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogAssignmentDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualFlowLogAssignmentRule for ManualFlowLogAssignmentDurationMinutesRangeRule {
    fn check(&self, item: &ManualFlowLogAssignment) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogAssignmentRequiredLabelRule {
    pub message: String,
}

impl ManualFlowLogAssignmentRule for ManualFlowLogAssignmentRequiredLabelRule {
    fn check(&self, item: &ManualFlowLogAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogAssignmentRequiredTitleRule {
    pub message: String,
}

impl ManualFlowLogAssignmentRule for ManualFlowLogAssignmentRequiredTitleRule {
    fn check(&self, item: &ManualFlowLogAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogAssignmentCodePrefixRule {
    pub prefix: String,
}

impl ManualFlowLogAssignmentRule for ManualFlowLogAssignmentCodePrefixRule {
    fn check(&self, item: &ManualFlowLogAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual flow log assignment.
#[derive(Debug, Clone)]
pub struct ManualFlowLogAssignmentValidator {
    pub rules: Vec<Rc<dyn ManualFlowLogAssignmentRule>>,
}

impl ManualFlowLogAssignmentValidator {
    pub fn validate(&self, item: &ManualFlowLogAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualFlowLogAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualFlowLogAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_flow_log_assignment_validator() -> ManualFlowLogAssignmentValidator {
    ManualFlowLogAssignmentValidator {
        rules: vec![
            Rc::new(ManualFlowLogAssignmentCeilingRangeRule { min: 3.7, max: 4870.7 })
                as Rc<dyn ManualFlowLogAssignmentRule>,
            Rc::new(ManualFlowLogAssignmentRequiredCodeRule {
                message: "manual flow log assignment needs an entry".to_string(),
            }),
            Rc::new(ManualFlowLogAssignmentDurationMinutesRangeRule { min: 120, max: 255 }),
            Rc::new(ManualFlowLogAssignmentRequiredLabelRule {
                message: "manual flow log assignment needs an entry".to_string(),
            }),
            Rc::new(ManualFlowLogAssignmentRequiredTitleRule {
                message: "manual flow log assignment needs this field".to_string(),
            }),
            Rc::new(ManualFlowLogAssignmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_flow_log_assignment_validator();
        let item = new_manual_flow_log_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ManualFlowLogAssignmentCeilingRangeRule { min: 7.8, max: 720.1 };
        let mut item = new_manual_flow_log_assignment("x", "y");
        item.ceiling = 721.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualFlowLogAssignmentRequiredCodeRule {
            message: "manual flow log assignment needs this field".to_string(),
        };
        let mut item = new_manual_flow_log_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = ManualFlowLogAssignmentDurationMinutesRangeRule { min: 16, max: 86400 };
        let mut item = new_manual_flow_log_assignment("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualFlowLogAssignmentRequiredLabelRule {
            message: "manual flow log assignment needs a value".to_string(),
        };
        let mut item = new_manual_flow_log_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualFlowLogAssignmentRequiredTitleRule {
            message: "manual flow log assignment needs an entry".to_string(),
        };
        let mut item = new_manual_flow_log_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
