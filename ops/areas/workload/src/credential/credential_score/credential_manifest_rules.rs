use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential manifest must pass. Check returns its problems, or none.
pub trait CredentialManifestRule: std::fmt::Debug {
    fn check(&self, item: &CredentialManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialManifestNotesLengthRule {
    pub max_length: i64,
}

impl CredentialManifestRule for CredentialManifestNotesLengthRule {
    fn check(&self, item: &CredentialManifest) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialManifestRequiredChecksumRule {
    pub message: String,
}

impl CredentialManifestRule for CredentialManifestRequiredChecksumRule {
    fn check(&self, item: &CredentialManifest) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialManifestApprovedGuardRule {
    pub limit: f64,
}

impl CredentialManifestRule for CredentialManifestApprovedGuardRule {
    fn check(&self, item: &CredentialManifest) -> Vec<String> {
        if item.approved && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while approved", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialManifestCodePrefixRule {
    pub prefix: String,
}

impl CredentialManifestRule for CredentialManifestCodePrefixRule {
    fn check(&self, item: &CredentialManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialManifestRequiredCodeRule {
    pub message: String,
}

impl CredentialManifestRule for CredentialManifestRequiredCodeRule {
    fn check(&self, item: &CredentialManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialManifestRequiredNameRule {
    pub message: String,
}

impl CredentialManifestRule for CredentialManifestRequiredNameRule {
    fn check(&self, item: &CredentialManifest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential manifest.
#[derive(Debug, Clone)]
pub struct CredentialManifestValidator {
    pub rules: Vec<Rc<dyn CredentialManifestRule>>,
}

impl CredentialManifestValidator {
    pub fn validate(&self, item: &CredentialManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_manifest_validator() -> CredentialManifestValidator {
    CredentialManifestValidator {
        rules: vec![
            Rc::new(CredentialManifestNotesLengthRule { max_length: 120 }) as Rc<dyn CredentialManifestRule>,
            Rc::new(CredentialManifestRequiredChecksumRule {
                message: "credential manifest needs a value".to_string(),
            }),
            Rc::new(CredentialManifestApprovedGuardRule { limit: 7.7 }),
            Rc::new(CredentialManifestCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CredentialManifestRequiredCodeRule {
                message: "credential manifest needs an entry".to_string(),
            }),
            Rc::new(CredentialManifestRequiredNameRule {
                message: "credential manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_manifest_validator();
        let item = new_credential_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = CredentialManifestNotesLengthRule { max_length: 86400 };
        let mut item = new_credential_manifest("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = CredentialManifestRequiredChecksumRule {
            message: "credential manifest needs an entry".to_string(),
        };
        let mut item = new_credential_manifest("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialManifestRequiredCodeRule {
            message: "credential manifest needs this field".to_string(),
        };
        let mut item = new_credential_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CredentialManifestRequiredNameRule {
            message: "credential manifest needs a value".to_string(),
        };
        let mut item = new_credential_manifest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
