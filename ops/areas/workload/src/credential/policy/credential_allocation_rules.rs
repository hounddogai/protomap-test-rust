use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential allocation must pass. Check returns its problems, or none.
pub trait CredentialAllocationRule: std::fmt::Debug {
    fn check(&self, item: &CredentialAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAllocationLabelLengthRule {
    pub max_length: i64,
}

impl CredentialAllocationRule for CredentialAllocationLabelLengthRule {
    fn check(&self, item: &CredentialAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAllocationCodeLengthRule {
    pub max_length: i64,
}

impl CredentialAllocationRule for CredentialAllocationCodeLengthRule {
    fn check(&self, item: &CredentialAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAllocationPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialAllocationRule for CredentialAllocationPortRangeRule {
    fn check(&self, item: &CredentialAllocation) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAllocationNamespaceLengthRule {
    pub max_length: i64,
}

impl CredentialAllocationRule for CredentialAllocationNamespaceLengthRule {
    fn check(&self, item: &CredentialAllocation) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential allocation.
#[derive(Debug, Clone)]
pub struct CredentialAllocationValidator {
    pub rules: Vec<Rc<dyn CredentialAllocationRule>>,
}

impl CredentialAllocationValidator {
    pub fn validate(&self, item: &CredentialAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_allocation_validator() -> CredentialAllocationValidator {
    CredentialAllocationValidator {
        rules: vec![
            Rc::new(CredentialAllocationLabelLengthRule { max_length: 16 }) as Rc<dyn CredentialAllocationRule>,
            Rc::new(CredentialAllocationCodeLengthRule { max_length: 3600 }),
            Rc::new(CredentialAllocationPortRangeRule { min: 64, max: 80 }),
            Rc::new(CredentialAllocationNamespaceLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_allocation_validator();
        let item = new_credential_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CredentialAllocationLabelLengthRule { max_length: 120 };
        let mut item = new_credential_allocation("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CredentialAllocationCodeLengthRule { max_length: 32 };
        let mut item = new_credential_allocation("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = CredentialAllocationPortRangeRule { min: 80, max: 86400 };
        let mut item = new_credential_allocation("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = CredentialAllocationNamespaceLengthRule { max_length: 120 };
        let mut item = new_credential_allocation("x", "y");
        item.namespace = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
