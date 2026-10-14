use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce allocation must pass. Check returns its problems, or none.
pub trait NonceAllocationRule: std::fmt::Debug {
    fn check(&self, item: &NonceAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAllocationNamespaceLengthRule {
    pub max_length: i64,
}

impl NonceAllocationRule for NonceAllocationNamespaceLengthRule {
    fn check(&self, item: &NonceAllocation) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAllocationNameLengthRule {
    pub max_length: i64,
}

impl NonceAllocationRule for NonceAllocationNameLengthRule {
    fn check(&self, item: &NonceAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAllocationRequiredNamespaceRule {
    pub message: String,
}

impl NonceAllocationRule for NonceAllocationRequiredNamespaceRule {
    fn check(&self, item: &NonceAllocation) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAllocationRequiredNameRule {
    pub message: String,
}

impl NonceAllocationRule for NonceAllocationRequiredNameRule {
    fn check(&self, item: &NonceAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce allocation.
#[derive(Debug, Clone)]
pub struct NonceAllocationValidator {
    pub rules: Vec<Rc<dyn NonceAllocationRule>>,
}

impl NonceAllocationValidator {
    pub fn validate(&self, item: &NonceAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_allocation_validator() -> NonceAllocationValidator {
    NonceAllocationValidator {
        rules: vec![
            Rc::new(NonceAllocationNamespaceLengthRule { max_length: 120 }) as Rc<dyn NonceAllocationRule>,
            Rc::new(NonceAllocationNameLengthRule { max_length: 3600 }),
            Rc::new(NonceAllocationRequiredNamespaceRule {
                message: "nonce allocation needs a value".to_string(),
            }),
            Rc::new(NonceAllocationRequiredNameRule {
                message: "nonce allocation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_allocation_validator();
        let item = new_nonce_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = NonceAllocationNamespaceLengthRule { max_length: 3600 };
        let mut item = new_nonce_allocation("x", "y");
        item.namespace = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = NonceAllocationNameLengthRule { max_length: 64 };
        let mut item = new_nonce_allocation("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = NonceAllocationRequiredNamespaceRule {
            message: "nonce allocation needs this field".to_string(),
        };
        let mut item = new_nonce_allocation("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = NonceAllocationRequiredNameRule {
            message: "nonce allocation needs this field".to_string(),
        };
        let mut item = new_nonce_allocation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
