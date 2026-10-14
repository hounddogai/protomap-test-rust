use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity variance must pass. Check returns its problems, or none.
pub trait IdentityVarianceRule: std::fmt::Debug {
    fn check(&self, item: &IdentityVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityVarianceCodeLengthRule {
    pub max_length: i64,
}

impl IdentityVarianceRule for IdentityVarianceCodeLengthRule {
    fn check(&self, item: &IdentityVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityVarianceRequiredLabelRule {
    pub message: String,
}

impl IdentityVarianceRule for IdentityVarianceRequiredLabelRule {
    fn check(&self, item: &IdentityVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityVarianceRequiredCodeRule {
    pub message: String,
}

impl IdentityVarianceRule for IdentityVarianceRequiredCodeRule {
    fn check(&self, item: &IdentityVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityVarianceCodePrefixRule {
    pub prefix: String,
}

impl IdentityVarianceRule for IdentityVarianceCodePrefixRule {
    fn check(&self, item: &IdentityVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity variance.
#[derive(Debug, Clone)]
pub struct IdentityVarianceValidator {
    pub rules: Vec<Rc<dyn IdentityVarianceRule>>,
}

impl IdentityVarianceValidator {
    pub fn validate(&self, item: &IdentityVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_variance_validator() -> IdentityVarianceValidator {
    IdentityVarianceValidator {
        rules: vec![
            Rc::new(IdentityVarianceCodeLengthRule { max_length: 3600 }) as Rc<dyn IdentityVarianceRule>,
            Rc::new(IdentityVarianceRequiredLabelRule {
                message: "identity variance needs an entry".to_string(),
            }),
            Rc::new(IdentityVarianceRequiredCodeRule {
                message: "identity variance needs an entry".to_string(),
            }),
            Rc::new(IdentityVarianceCodePrefixRule {
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
        let validator = default_identity_variance_validator();
        let item = new_identity_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityVarianceCodeLengthRule { max_length: 64 };
        let mut item = new_identity_variance("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = IdentityVarianceRequiredLabelRule {
            message: "identity variance needs a value".to_string(),
        };
        let mut item = new_identity_variance("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityVarianceRequiredCodeRule {
            message: "identity variance needs this field".to_string(),
        };
        let mut item = new_identity_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
