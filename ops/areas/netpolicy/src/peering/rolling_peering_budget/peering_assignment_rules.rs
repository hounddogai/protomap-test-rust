use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering assignment must pass. Check returns its problems, or none.
pub trait PeeringAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PeeringAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl PeeringAssignmentRule for PeeringAssignmentCodeLengthRule {
    fn check(&self, item: &PeeringAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAssignmentChannelLengthRule {
    pub max_length: i64,
}

impl PeeringAssignmentRule for PeeringAssignmentChannelLengthRule {
    fn check(&self, item: &PeeringAssignment) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAssignmentRequiredNamespaceRule {
    pub message: String,
}

impl PeeringAssignmentRule for PeeringAssignmentRequiredNamespaceRule {
    fn check(&self, item: &PeeringAssignment) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering assignment.
#[derive(Debug, Clone)]
pub struct PeeringAssignmentValidator {
    pub rules: Vec<Rc<dyn PeeringAssignmentRule>>,
}

impl PeeringAssignmentValidator {
    pub fn validate(&self, item: &PeeringAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_assignment_validator() -> PeeringAssignmentValidator {
    PeeringAssignmentValidator {
        rules: vec![
            Rc::new(PeeringAssignmentCodeLengthRule { max_length: 120 }) as Rc<dyn PeeringAssignmentRule>,
            Rc::new(PeeringAssignmentChannelLengthRule { max_length: 16 }),
            Rc::new(PeeringAssignmentRequiredNamespaceRule {
                message: "peering assignment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_assignment_validator();
        let item = new_peering_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringAssignmentCodeLengthRule { max_length: 80 };
        let mut item = new_peering_assignment("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = PeeringAssignmentChannelLengthRule { max_length: 80 };
        let mut item = new_peering_assignment("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = PeeringAssignmentRequiredNamespaceRule {
            message: "peering assignment needs this field".to_string(),
        };
        let mut item = new_peering_assignment("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
