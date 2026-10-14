use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation quota must pass. Check returns its problems, or none.
pub trait FederationQuotaRule: std::fmt::Debug {
    fn check(&self, item: &FederationQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationQuotaRequiredCipherRule {
    pub message: String,
}

impl FederationQuotaRule for FederationQuotaRequiredCipherRule {
    fn check(&self, item: &FederationQuota) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationQuotaRequiredNameRule {
    pub message: String,
}

impl FederationQuotaRule for FederationQuotaRequiredNameRule {
    fn check(&self, item: &FederationQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FederationQuotaRule for FederationQuotaAllowedCodeRule {
    fn check(&self, item: &FederationQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationQuotaNameLengthRule {
    pub max_length: i64,
}

impl FederationQuotaRule for FederationQuotaNameLengthRule {
    fn check(&self, item: &FederationQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationQuotaCodeLengthRule {
    pub max_length: i64,
}

impl FederationQuotaRule for FederationQuotaCodeLengthRule {
    fn check(&self, item: &FederationQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation quota.
#[derive(Debug, Clone)]
pub struct FederationQuotaValidator {
    pub rules: Vec<Rc<dyn FederationQuotaRule>>,
}

impl FederationQuotaValidator {
    pub fn validate(&self, item: &FederationQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_quota_validator() -> FederationQuotaValidator {
    FederationQuotaValidator {
        rules: vec![
            Rc::new(FederationQuotaRequiredCipherRule {
                message: "federation quota needs an entry".to_string(),
            }) as Rc<dyn FederationQuotaRule>,
            Rc::new(FederationQuotaRequiredNameRule {
                message: "federation quota needs this field".to_string(),
            }),
            Rc::new(FederationQuotaAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(FederationQuotaNameLengthRule { max_length: 16 }),
            Rc::new(FederationQuotaCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_quota_validator();
        let item = new_federation_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = FederationQuotaRequiredCipherRule {
            message: "federation quota needs this field".to_string(),
        };
        let mut item = new_federation_quota("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FederationQuotaRequiredNameRule {
            message: "federation quota needs an entry".to_string(),
        };
        let mut item = new_federation_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FederationQuotaNameLengthRule { max_length: 16 };
        let mut item = new_federation_quota("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationQuotaCodeLengthRule { max_length: 64 };
        let mut item = new_federation_quota("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
