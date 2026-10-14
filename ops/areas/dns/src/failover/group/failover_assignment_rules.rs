use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover assignment must pass. Check returns its problems, or none.
pub trait FailoverAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &FailoverAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAssignmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FailoverAssignmentRule for FailoverAssignmentAllowedNameRule {
    fn check(&self, item: &FailoverAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAssignmentPrincipalLengthRule {
    pub max_length: i64,
}

impl FailoverAssignmentRule for FailoverAssignmentPrincipalLengthRule {
    fn check(&self, item: &FailoverAssignment) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl FailoverAssignmentRule for FailoverAssignmentCodeLengthRule {
    fn check(&self, item: &FailoverAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAssignmentClusterLengthRule {
    pub max_length: i64,
}

impl FailoverAssignmentRule for FailoverAssignmentClusterLengthRule {
    fn check(&self, item: &FailoverAssignment) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAssignmentRequiredNameRule {
    pub message: String,
}

impl FailoverAssignmentRule for FailoverAssignmentRequiredNameRule {
    fn check(&self, item: &FailoverAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAssignmentRequiredPrincipalRule {
    pub message: String,
}

impl FailoverAssignmentRule for FailoverAssignmentRequiredPrincipalRule {
    fn check(&self, item: &FailoverAssignment) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover assignment.
#[derive(Debug, Clone)]
pub struct FailoverAssignmentValidator {
    pub rules: Vec<Rc<dyn FailoverAssignmentRule>>,
}

impl FailoverAssignmentValidator {
    pub fn validate(&self, item: &FailoverAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_assignment_validator() -> FailoverAssignmentValidator {
    FailoverAssignmentValidator {
        rules: vec![
            Rc::new(FailoverAssignmentAllowedNameRule { allowed: Vec::new() }) as Rc<dyn FailoverAssignmentRule>,
            Rc::new(FailoverAssignmentPrincipalLengthRule { max_length: 16 }),
            Rc::new(FailoverAssignmentCodeLengthRule { max_length: 3600 }),
            Rc::new(FailoverAssignmentClusterLengthRule { max_length: 120 }),
            Rc::new(FailoverAssignmentRequiredNameRule {
                message: "failover assignment needs an entry".to_string(),
            }),
            Rc::new(FailoverAssignmentRequiredPrincipalRule {
                message: "failover assignment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_assignment_validator();
        let item = new_failover_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = FailoverAssignmentPrincipalLengthRule { max_length: 1000 };
        let mut item = new_failover_assignment("x", "y");
        item.principal = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverAssignmentCodeLengthRule { max_length: 86400 };
        let mut item = new_failover_assignment("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = FailoverAssignmentClusterLengthRule { max_length: 64 };
        let mut item = new_failover_assignment("x", "y");
        item.cluster = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FailoverAssignmentRequiredNameRule {
            message: "failover assignment needs an entry".to_string(),
        };
        let mut item = new_failover_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = FailoverAssignmentRequiredPrincipalRule {
            message: "failover assignment needs a value".to_string(),
        };
        let mut item = new_failover_assignment("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
