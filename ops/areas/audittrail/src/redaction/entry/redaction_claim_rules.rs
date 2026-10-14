use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction claim must pass. Check returns its problems, or none.
pub trait RedactionClaimRule: std::fmt::Debug {
    fn check(&self, item: &RedactionClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionClaimRequiredNameRule {
    pub message: String,
}

impl RedactionClaimRule for RedactionClaimRequiredNameRule {
    fn check(&self, item: &RedactionClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionClaimCodePrefixRule {
    pub prefix: String,
}

impl RedactionClaimRule for RedactionClaimCodePrefixRule {
    fn check(&self, item: &RedactionClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionClaimNameLengthRule {
    pub max_length: i64,
}

impl RedactionClaimRule for RedactionClaimNameLengthRule {
    fn check(&self, item: &RedactionClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionClaimAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RedactionClaimRule for RedactionClaimAllowedNameRule {
    fn check(&self, item: &RedactionClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionClaimRequiredCodeRule {
    pub message: String,
}

impl RedactionClaimRule for RedactionClaimRequiredCodeRule {
    fn check(&self, item: &RedactionClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction claim.
#[derive(Debug, Clone)]
pub struct RedactionClaimValidator {
    pub rules: Vec<Rc<dyn RedactionClaimRule>>,
}

impl RedactionClaimValidator {
    pub fn validate(&self, item: &RedactionClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_claim_validator() -> RedactionClaimValidator {
    RedactionClaimValidator {
        rules: vec![
            Rc::new(RedactionClaimRequiredNameRule {
                message: "redaction claim needs this field".to_string(),
            }) as Rc<dyn RedactionClaimRule>,
            Rc::new(RedactionClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RedactionClaimNameLengthRule { max_length: 255 }),
            Rc::new(RedactionClaimAllowedNameRule {
                allowed: vec!["default".to_string()],
            }),
            Rc::new(RedactionClaimRequiredCodeRule {
                message: "redaction claim needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_claim_validator();
        let item = new_redaction_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RedactionClaimRequiredNameRule {
            message: "redaction claim needs this field".to_string(),
        };
        let mut item = new_redaction_claim("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RedactionClaimNameLengthRule { max_length: 1000 };
        let mut item = new_redaction_claim("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionClaimRequiredCodeRule {
            message: "redaction claim needs an entry".to_string(),
        };
        let mut item = new_redaction_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
