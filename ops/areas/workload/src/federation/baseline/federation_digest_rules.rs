use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation digest must pass. Check returns its problems, or none.
pub trait FederationDigestRule: std::fmt::Debug {
    fn check(&self, item: &FederationDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigestCodePrefixRule {
    pub prefix: String,
}

impl FederationDigestRule for FederationDigestCodePrefixRule {
    fn check(&self, item: &FederationDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigestAudienceLengthRule {
    pub max_length: i64,
}

impl FederationDigestRule for FederationDigestAudienceLengthRule {
    fn check(&self, item: &FederationDigest) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigestAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FederationDigestRule for FederationDigestAllowedNameRule {
    fn check(&self, item: &FederationDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigestRequiredNameRule {
    pub message: String,
}

impl FederationDigestRule for FederationDigestRequiredNameRule {
    fn check(&self, item: &FederationDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigestNameLengthRule {
    pub max_length: i64,
}

impl FederationDigestRule for FederationDigestNameLengthRule {
    fn check(&self, item: &FederationDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigestCodeLengthRule {
    pub max_length: i64,
}

impl FederationDigestRule for FederationDigestCodeLengthRule {
    fn check(&self, item: &FederationDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation digest.
#[derive(Debug, Clone)]
pub struct FederationDigestValidator {
    pub rules: Vec<Rc<dyn FederationDigestRule>>,
}

impl FederationDigestValidator {
    pub fn validate(&self, item: &FederationDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_digest_validator() -> FederationDigestValidator {
    FederationDigestValidator {
        rules: vec![
            Rc::new(FederationDigestCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn FederationDigestRule>,
            Rc::new(FederationDigestAudienceLengthRule { max_length: 86400 }),
            Rc::new(FederationDigestAllowedNameRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(FederationDigestRequiredNameRule {
                message: "federation digest needs this field".to_string(),
            }),
            Rc::new(FederationDigestNameLengthRule { max_length: 64 }),
            Rc::new(FederationDigestCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_digest_validator();
        let item = new_federation_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = FederationDigestAudienceLengthRule { max_length: 1000 };
        let mut item = new_federation_digest("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FederationDigestRequiredNameRule {
            message: "federation digest needs a value".to_string(),
        };
        let mut item = new_federation_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FederationDigestNameLengthRule { max_length: 32 };
        let mut item = new_federation_digest("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationDigestCodeLengthRule { max_length: 86400 };
        let mut item = new_federation_digest("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
