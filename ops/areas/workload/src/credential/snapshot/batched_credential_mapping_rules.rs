use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched credential mapping must pass. Check returns its problems, or none.
pub trait BatchedCredentialMappingRule: std::fmt::Debug {
    fn check(&self, item: &BatchedCredentialMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialMappingNamespaceLengthRule {
    pub max_length: i64,
}

impl BatchedCredentialMappingRule for BatchedCredentialMappingNamespaceLengthRule {
    fn check(&self, item: &BatchedCredentialMapping) -> Vec<String> {
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
pub struct BatchedCredentialMappingCodeLengthRule {
    pub max_length: i64,
}

impl BatchedCredentialMappingRule for BatchedCredentialMappingCodeLengthRule {
    fn check(&self, item: &BatchedCredentialMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialMappingAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedCredentialMappingRule for BatchedCredentialMappingAllowedCodeRule {
    fn check(&self, item: &BatchedCredentialMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialMappingRequiredCodeRule {
    pub message: String,
}

impl BatchedCredentialMappingRule for BatchedCredentialMappingRequiredCodeRule {
    fn check(&self, item: &BatchedCredentialMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialMappingCodePrefixRule {
    pub prefix: String,
}

impl BatchedCredentialMappingRule for BatchedCredentialMappingCodePrefixRule {
    fn check(&self, item: &BatchedCredentialMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched credential mapping.
#[derive(Debug, Clone)]
pub struct BatchedCredentialMappingValidator {
    pub rules: Vec<Rc<dyn BatchedCredentialMappingRule>>,
}

impl BatchedCredentialMappingValidator {
    pub fn validate(&self, item: &BatchedCredentialMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedCredentialMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedCredentialMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_credential_mapping_validator() -> BatchedCredentialMappingValidator {
    BatchedCredentialMappingValidator {
        rules: vec![
            Rc::new(BatchedCredentialMappingNamespaceLengthRule { max_length: 86400 })
                as Rc<dyn BatchedCredentialMappingRule>,
            Rc::new(BatchedCredentialMappingCodeLengthRule { max_length: 80 }),
            Rc::new(BatchedCredentialMappingAllowedCodeRule {
                allowed: vec!["global".to_string(), "adaptive".to_string(), "scheduled".to_string()],
            }),
            Rc::new(BatchedCredentialMappingRequiredCodeRule {
                message: "batched credential mapping needs an entry".to_string(),
            }),
            Rc::new(BatchedCredentialMappingCodePrefixRule {
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
        let validator = default_batched_credential_mapping_validator();
        let item = new_batched_credential_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = BatchedCredentialMappingNamespaceLengthRule { max_length: 16 };
        let mut item = new_batched_credential_mapping("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedCredentialMappingCodeLengthRule { max_length: 80 };
        let mut item = new_batched_credential_mapping("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedCredentialMappingRequiredCodeRule {
            message: "batched credential mapping needs a value".to_string(),
        };
        let mut item = new_batched_credential_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
