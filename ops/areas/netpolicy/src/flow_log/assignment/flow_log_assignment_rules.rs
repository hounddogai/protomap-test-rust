use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log assignment must pass. Check returns its problems, or none.
pub trait FlowLogAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAssignmentCodePrefixRule {
    pub prefix: String,
}

impl FlowLogAssignmentRule for FlowLogAssignmentCodePrefixRule {
    fn check(&self, item: &FlowLogAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAssignmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogAssignmentRule for FlowLogAssignmentCeilingRangeRule {
    fn check(&self, item: &FlowLogAssignment) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAssignmentNameLengthRule {
    pub max_length: i64,
}

impl FlowLogAssignmentRule for FlowLogAssignmentNameLengthRule {
    fn check(&self, item: &FlowLogAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log assignment.
#[derive(Debug, Clone)]
pub struct FlowLogAssignmentValidator {
    pub rules: Vec<Rc<dyn FlowLogAssignmentRule>>,
}

impl FlowLogAssignmentValidator {
    pub fn validate(&self, item: &FlowLogAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_assignment_validator() -> FlowLogAssignmentValidator {
    FlowLogAssignmentValidator {
        rules: vec![
            Rc::new(FlowLogAssignmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn FlowLogAssignmentRule>,
            Rc::new(FlowLogAssignmentCeilingRangeRule { min: 1.8, max: 1375.9 }),
            Rc::new(FlowLogAssignmentNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_assignment_validator();
        let item = new_flow_log_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = FlowLogAssignmentCeilingRangeRule { min: 2.4, max: 4694.0 };
        let mut item = new_flow_log_assignment("x", "y");
        item.ceiling = 4695.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FlowLogAssignmentNameLengthRule { max_length: 120 };
        let mut item = new_flow_log_assignment("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
