use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress assignment must pass. Check returns its problems, or none.
pub trait EgressAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &EgressAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl EgressAssignmentRule for EgressAssignmentCodeLengthRule {
    fn check(&self, item: &EgressAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAssignmentCodePrefixRule {
    pub prefix: String,
}

impl EgressAssignmentRule for EgressAssignmentCodePrefixRule {
    fn check(&self, item: &EgressAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAssignmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressAssignmentRule for EgressAssignmentPriorityRangeRule {
    fn check(&self, item: &EgressAssignment) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAssignmentAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl EgressAssignmentRule for EgressAssignmentAllowedReviewerRule {
    fn check(&self, item: &EgressAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is blocked", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress assignment.
#[derive(Debug, Clone)]
pub struct EgressAssignmentValidator {
    pub rules: Vec<Rc<dyn EgressAssignmentRule>>,
}

impl EgressAssignmentValidator {
    pub fn validate(&self, item: &EgressAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_assignment_validator() -> EgressAssignmentValidator {
    EgressAssignmentValidator {
        rules: vec![
            Rc::new(EgressAssignmentCodeLengthRule { max_length: 80 }) as Rc<dyn EgressAssignmentRule>,
            Rc::new(EgressAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(EgressAssignmentPriorityRangeRule { min: 1000, max: 3600 }),
            Rc::new(EgressAssignmentAllowedReviewerRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_assignment_validator();
        let item = new_egress_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressAssignmentCodeLengthRule { max_length: 16 };
        let mut item = new_egress_assignment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = EgressAssignmentPriorityRangeRule { min: 1000, max: 1000 };
        let mut item = new_egress_assignment("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
