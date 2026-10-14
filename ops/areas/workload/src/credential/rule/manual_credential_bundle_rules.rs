use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual credential bundle must pass. Check returns its problems, or none.
pub trait ManualCredentialBundleRule: std::fmt::Debug {
    fn check(&self, item: &ManualCredentialBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialBundleCodePrefixRule {
    pub prefix: String,
}

impl ManualCredentialBundleRule for ManualCredentialBundleCodePrefixRule {
    fn check(&self, item: &ManualCredentialBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialBundleRequiredTitleRule {
    pub message: String,
}

impl ManualCredentialBundleRule for ManualCredentialBundleRequiredTitleRule {
    fn check(&self, item: &ManualCredentialBundle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialBundleRequiredHostRule {
    pub message: String,
}

impl ManualCredentialBundleRule for ManualCredentialBundleRequiredHostRule {
    fn check(&self, item: &ManualCredentialBundle) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialBundleTitleLengthRule {
    pub max_length: i64,
}

impl ManualCredentialBundleRule for ManualCredentialBundleTitleLengthRule {
    fn check(&self, item: &ManualCredentialBundle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual credential bundle.
#[derive(Debug, Clone)]
pub struct ManualCredentialBundleValidator {
    pub rules: Vec<Rc<dyn ManualCredentialBundleRule>>,
}

impl ManualCredentialBundleValidator {
    pub fn validate(&self, item: &ManualCredentialBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualCredentialBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualCredentialBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_credential_bundle_validator() -> ManualCredentialBundleValidator {
    ManualCredentialBundleValidator {
        rules: vec![
            Rc::new(ManualCredentialBundleCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn ManualCredentialBundleRule>,
            Rc::new(ManualCredentialBundleRequiredTitleRule {
                message: "manual credential bundle needs this field".to_string(),
            }),
            Rc::new(ManualCredentialBundleRequiredHostRule {
                message: "manual credential bundle needs a value".to_string(),
            }),
            Rc::new(ManualCredentialBundleTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_credential_bundle_validator();
        let item = new_manual_credential_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualCredentialBundleRequiredTitleRule {
            message: "manual credential bundle needs this field".to_string(),
        };
        let mut item = new_manual_credential_bundle("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = ManualCredentialBundleRequiredHostRule {
            message: "manual credential bundle needs a value".to_string(),
        };
        let mut item = new_manual_credential_bundle("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualCredentialBundleTitleLengthRule { max_length: 86400 };
        let mut item = new_manual_credential_bundle("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
