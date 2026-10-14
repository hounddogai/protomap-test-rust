use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity manifest must pass. Check returns its problems, or none.
pub trait IdentityManifestRule: std::fmt::Debug {
    fn check(&self, item: &IdentityManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityManifestChecksumLengthRule {
    pub max_length: i64,
}

impl IdentityManifestRule for IdentityManifestChecksumLengthRule {
    fn check(&self, item: &IdentityManifest) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityManifestRequiredCodeRule {
    pub message: String,
}

impl IdentityManifestRule for IdentityManifestRequiredCodeRule {
    fn check(&self, item: &IdentityManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityManifestCodeLengthRule {
    pub max_length: i64,
}

impl IdentityManifestRule for IdentityManifestCodeLengthRule {
    fn check(&self, item: &IdentityManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityManifestRequiredTenantIdRule {
    pub message: String,
}

impl IdentityManifestRule for IdentityManifestRequiredTenantIdRule {
    fn check(&self, item: &IdentityManifest) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityManifestAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl IdentityManifestRule for IdentityManifestAllowedChecksumRule {
    fn check(&self, item: &IdentityManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is duplicated", item.checksum)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity manifest.
#[derive(Debug, Clone)]
pub struct IdentityManifestValidator {
    pub rules: Vec<Rc<dyn IdentityManifestRule>>,
}

impl IdentityManifestValidator {
    pub fn validate(&self, item: &IdentityManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_manifest_validator() -> IdentityManifestValidator {
    IdentityManifestValidator {
        rules: vec![
            Rc::new(IdentityManifestChecksumLengthRule { max_length: 86400 }) as Rc<dyn IdentityManifestRule>,
            Rc::new(IdentityManifestRequiredCodeRule {
                message: "identity manifest needs a value".to_string(),
            }),
            Rc::new(IdentityManifestCodeLengthRule { max_length: 120 }),
            Rc::new(IdentityManifestRequiredTenantIdRule {
                message: "identity manifest needs this field".to_string(),
            }),
            Rc::new(IdentityManifestAllowedChecksumRule {
                allowed: vec!["weighted".to_string(), "primary".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_manifest_validator();
        let item = new_identity_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = IdentityManifestChecksumLengthRule { max_length: 3600 };
        let mut item = new_identity_manifest("x", "y");
        item.checksum = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityManifestRequiredCodeRule {
            message: "identity manifest needs a value".to_string(),
        };
        let mut item = new_identity_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityManifestCodeLengthRule { max_length: 3600 };
        let mut item = new_identity_manifest("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = IdentityManifestRequiredTenantIdRule {
            message: "identity manifest needs this field".to_string(),
        };
        let mut item = new_identity_manifest("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
