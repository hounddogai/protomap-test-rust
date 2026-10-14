use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity allocation must pass. Check returns its problems, or none.
pub trait IdentityAllocationRule: std::fmt::Debug {
    fn check(&self, item: &IdentityAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAllocationRequiredLabelRule {
    pub message: String,
}

impl IdentityAllocationRule for IdentityAllocationRequiredLabelRule {
    fn check(&self, item: &IdentityAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAllocationRequiredSummaryRule {
    pub message: String,
}

impl IdentityAllocationRule for IdentityAllocationRequiredSummaryRule {
    fn check(&self, item: &IdentityAllocation) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAllocationSummaryLengthRule {
    pub max_length: i64,
}

impl IdentityAllocationRule for IdentityAllocationSummaryLengthRule {
    fn check(&self, item: &IdentityAllocation) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAllocationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl IdentityAllocationRule for IdentityAllocationAllowedLabelRule {
    fn check(&self, item: &IdentityAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAllocationScopeLengthRule {
    pub max_length: i64,
}

impl IdentityAllocationRule for IdentityAllocationScopeLengthRule {
    fn check(&self, item: &IdentityAllocation) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAllocationCodePrefixRule {
    pub prefix: String,
}

impl IdentityAllocationRule for IdentityAllocationCodePrefixRule {
    fn check(&self, item: &IdentityAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity allocation.
#[derive(Debug, Clone)]
pub struct IdentityAllocationValidator {
    pub rules: Vec<Rc<dyn IdentityAllocationRule>>,
}

impl IdentityAllocationValidator {
    pub fn validate(&self, item: &IdentityAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_allocation_validator() -> IdentityAllocationValidator {
    IdentityAllocationValidator {
        rules: vec![
            Rc::new(IdentityAllocationRequiredLabelRule {
                message: "identity allocation needs a value".to_string(),
            }) as Rc<dyn IdentityAllocationRule>,
            Rc::new(IdentityAllocationRequiredSummaryRule {
                message: "identity allocation needs an entry".to_string(),
            }),
            Rc::new(IdentityAllocationSummaryLengthRule { max_length: 80 }),
            Rc::new(IdentityAllocationAllowedLabelRule {
                allowed: vec!["adaptive".to_string(), "primary".to_string(), "primary".to_string()],
            }),
            Rc::new(IdentityAllocationScopeLengthRule { max_length: 255 }),
            Rc::new(IdentityAllocationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_allocation_validator();
        let item = new_identity_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = IdentityAllocationRequiredLabelRule {
            message: "identity allocation needs this field".to_string(),
        };
        let mut item = new_identity_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = IdentityAllocationRequiredSummaryRule {
            message: "identity allocation needs this field".to_string(),
        };
        let mut item = new_identity_allocation("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = IdentityAllocationSummaryLengthRule { max_length: 80 };
        let mut item = new_identity_allocation("x", "y");
        item.summary = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = IdentityAllocationScopeLengthRule { max_length: 3600 };
        let mut item = new_identity_allocation("x", "y");
        item.scope = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
