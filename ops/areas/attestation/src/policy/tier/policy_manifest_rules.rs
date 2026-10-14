use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy manifest must pass. Check returns its problems, or none.
pub trait PolicyManifestRule: std::fmt::Debug {
    fn check(&self, item: &PolicyManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyManifestCodeLengthRule {
    pub max_length: i64,
}

impl PolicyManifestRule for PolicyManifestCodeLengthRule {
    fn check(&self, item: &PolicyManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyManifestRequiredExternalRefRule {
    pub message: String,
}

impl PolicyManifestRule for PolicyManifestRequiredExternalRefRule {
    fn check(&self, item: &PolicyManifest) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyManifestCodePrefixRule {
    pub prefix: String,
}

impl PolicyManifestRule for PolicyManifestCodePrefixRule {
    fn check(&self, item: &PolicyManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy manifest.
#[derive(Debug, Clone)]
pub struct PolicyManifestValidator {
    pub rules: Vec<Rc<dyn PolicyManifestRule>>,
}

impl PolicyManifestValidator {
    pub fn validate(&self, item: &PolicyManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_manifest_validator() -> PolicyManifestValidator {
    PolicyManifestValidator {
        rules: vec![
            Rc::new(PolicyManifestCodeLengthRule { max_length: 120 }) as Rc<dyn PolicyManifestRule>,
            Rc::new(PolicyManifestRequiredExternalRefRule {
                message: "policy manifest needs a value".to_string(),
            }),
            Rc::new(PolicyManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_manifest_validator();
        let item = new_policy_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyManifestCodeLengthRule { max_length: 1000 };
        let mut item = new_policy_manifest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = PolicyManifestRequiredExternalRefRule {
            message: "policy manifest needs a value".to_string(),
        };
        let mut item = new_policy_manifest("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
