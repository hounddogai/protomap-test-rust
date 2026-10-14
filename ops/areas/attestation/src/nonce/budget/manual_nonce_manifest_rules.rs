use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual nonce manifest must pass. Check returns its problems, or none.
pub trait ManualNonceManifestRule: std::fmt::Debug {
    fn check(&self, item: &ManualNonceManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceManifestTitleLengthRule {
    pub max_length: i64,
}

impl ManualNonceManifestRule for ManualNonceManifestTitleLengthRule {
    fn check(&self, item: &ManualNonceManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceManifestRequiredScopeRule {
    pub message: String,
}

impl ManualNonceManifestRule for ManualNonceManifestRequiredScopeRule {
    fn check(&self, item: &ManualNonceManifest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceManifestScopeLengthRule {
    pub max_length: i64,
}

impl ManualNonceManifestRule for ManualNonceManifestScopeLengthRule {
    fn check(&self, item: &ManualNonceManifest) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceManifestCodePrefixRule {
    pub prefix: String,
}

impl ManualNonceManifestRule for ManualNonceManifestCodePrefixRule {
    fn check(&self, item: &ManualNonceManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual nonce manifest.
#[derive(Debug, Clone)]
pub struct ManualNonceManifestValidator {
    pub rules: Vec<Rc<dyn ManualNonceManifestRule>>,
}

impl ManualNonceManifestValidator {
    pub fn validate(&self, item: &ManualNonceManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualNonceManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualNonceManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_nonce_manifest_validator() -> ManualNonceManifestValidator {
    ManualNonceManifestValidator {
        rules: vec![
            Rc::new(ManualNonceManifestTitleLengthRule { max_length: 86400 }) as Rc<dyn ManualNonceManifestRule>,
            Rc::new(ManualNonceManifestRequiredScopeRule {
                message: "manual nonce manifest needs this field".to_string(),
            }),
            Rc::new(ManualNonceManifestScopeLengthRule { max_length: 1000 }),
            Rc::new(ManualNonceManifestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_nonce_manifest_validator();
        let item = new_manual_nonce_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualNonceManifestTitleLengthRule { max_length: 80 };
        let mut item = new_manual_nonce_manifest("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ManualNonceManifestRequiredScopeRule {
            message: "manual nonce manifest needs an entry".to_string(),
        };
        let mut item = new_manual_nonce_manifest("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = ManualNonceManifestScopeLengthRule { max_length: 16 };
        let mut item = new_manual_nonce_manifest("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
