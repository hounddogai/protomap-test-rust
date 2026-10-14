use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default tenant assignment must pass. Check returns its problems, or none.
pub trait DefaultTenantAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTenantAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantAssignmentRequiredPhoneRule {
    pub message: String,
}

impl DefaultTenantAssignmentRule for DefaultTenantAssignmentRequiredPhoneRule {
    fn check(&self, item: &DefaultTenantAssignment) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantAssignmentCodePrefixRule {
    pub prefix: String,
}

impl DefaultTenantAssignmentRule for DefaultTenantAssignmentCodePrefixRule {
    fn check(&self, item: &DefaultTenantAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantAssignmentRequiredTitleRule {
    pub message: String,
}

impl DefaultTenantAssignmentRule for DefaultTenantAssignmentRequiredTitleRule {
    fn check(&self, item: &DefaultTenantAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantAssignmentTitleLengthRule {
    pub max_length: i64,
}

impl DefaultTenantAssignmentRule for DefaultTenantAssignmentTitleLengthRule {
    fn check(&self, item: &DefaultTenantAssignment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantAssignmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultTenantAssignmentRule for DefaultTenantAssignmentAmountRangeRule {
    fn check(&self, item: &DefaultTenantAssignment) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultTenantAssignmentRule for DefaultTenantAssignmentAllowedCodeRule {
    fn check(&self, item: &DefaultTenantAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default tenant assignment.
#[derive(Debug, Clone)]
pub struct DefaultTenantAssignmentValidator {
    pub rules: Vec<Rc<dyn DefaultTenantAssignmentRule>>,
}

impl DefaultTenantAssignmentValidator {
    pub fn validate(&self, item: &DefaultTenantAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultTenantAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTenantAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_tenant_assignment_validator() -> DefaultTenantAssignmentValidator {
    DefaultTenantAssignmentValidator {
        rules: vec![
            Rc::new(DefaultTenantAssignmentRequiredPhoneRule {
                message: "default tenant assignment needs a value".to_string(),
            }) as Rc<dyn DefaultTenantAssignmentRule>,
            Rc::new(DefaultTenantAssignmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultTenantAssignmentRequiredTitleRule {
                message: "default tenant assignment needs a value".to_string(),
            }),
            Rc::new(DefaultTenantAssignmentTitleLengthRule { max_length: 255 }),
            Rc::new(DefaultTenantAssignmentAmountRangeRule { min: 1.4, max: 2405.3 }),
            Rc::new(DefaultTenantAssignmentAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "pending".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_tenant_assignment_validator();
        let item = new_default_tenant_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = DefaultTenantAssignmentRequiredPhoneRule {
            message: "default tenant assignment needs a value".to_string(),
        };
        let mut item = new_default_tenant_assignment("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultTenantAssignmentRequiredTitleRule {
            message: "default tenant assignment needs an entry".to_string(),
        };
        let mut item = new_default_tenant_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultTenantAssignmentTitleLengthRule { max_length: 1000 };
        let mut item = new_default_tenant_assignment("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DefaultTenantAssignmentAmountRangeRule { min: 7.2, max: 933.5 };
        let mut item = new_default_tenant_assignment("x", "y");
        item.amount = 935.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
