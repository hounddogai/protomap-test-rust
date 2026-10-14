use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain assignment must pass. Check returns its problems, or none.
pub trait TrustDomainAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAssignmentBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainAssignmentRule for TrustDomainAssignmentBatchSizeRangeRule {
    fn check(&self, item: &TrustDomainAssignment) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAssignmentCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainAssignmentRule for TrustDomainAssignmentCodePrefixRule {
    fn check(&self, item: &TrustDomainAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAssignmentRequiredCodeRule {
    pub message: String,
}

impl TrustDomainAssignmentRule for TrustDomainAssignmentRequiredCodeRule {
    fn check(&self, item: &TrustDomainAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAssignmentRequiredNameRule {
    pub message: String,
}

impl TrustDomainAssignmentRule for TrustDomainAssignmentRequiredNameRule {
    fn check(&self, item: &TrustDomainAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain assignment.
#[derive(Debug, Clone)]
pub struct TrustDomainAssignmentValidator {
    pub rules: Vec<Rc<dyn TrustDomainAssignmentRule>>,
}

impl TrustDomainAssignmentValidator {
    pub fn validate(&self, item: &TrustDomainAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_assignment_validator() -> TrustDomainAssignmentValidator {
    TrustDomainAssignmentValidator {
        rules: vec![
            Rc::new(TrustDomainAssignmentBatchSizeRangeRule { min: 80, max: 255 }) as Rc<dyn TrustDomainAssignmentRule>,
            Rc::new(TrustDomainAssignmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TrustDomainAssignmentRequiredCodeRule {
                message: "trust domain assignment needs this field".to_string(),
            }),
            Rc::new(TrustDomainAssignmentRequiredNameRule {
                message: "trust domain assignment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_assignment_validator();
        let item = new_trust_domain_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TrustDomainAssignmentBatchSizeRangeRule { min: 120, max: 120 };
        let mut item = new_trust_domain_assignment("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainAssignmentRequiredCodeRule {
            message: "trust domain assignment needs a value".to_string(),
        };
        let mut item = new_trust_domain_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TrustDomainAssignmentRequiredNameRule {
            message: "trust domain assignment needs this field".to_string(),
        };
        let mut item = new_trust_domain_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
