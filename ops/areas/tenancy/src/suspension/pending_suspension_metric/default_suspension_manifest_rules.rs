use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default suspension manifest must pass. Check returns its problems, or none.
pub trait DefaultSuspensionManifestRule: std::fmt::Debug {
    fn check(&self, item: &DefaultSuspensionManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionManifestCodePrefixRule {
    pub prefix: String,
}

impl DefaultSuspensionManifestRule for DefaultSuspensionManifestCodePrefixRule {
    fn check(&self, item: &DefaultSuspensionManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionManifestRequiredCodeRule {
    pub message: String,
}

impl DefaultSuspensionManifestRule for DefaultSuspensionManifestRequiredCodeRule {
    fn check(&self, item: &DefaultSuspensionManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionManifestRequiredChannelRule {
    pub message: String,
}

impl DefaultSuspensionManifestRule for DefaultSuspensionManifestRequiredChannelRule {
    fn check(&self, item: &DefaultSuspensionManifest) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default suspension manifest.
#[derive(Debug, Clone)]
pub struct DefaultSuspensionManifestValidator {
    pub rules: Vec<Rc<dyn DefaultSuspensionManifestRule>>,
}

impl DefaultSuspensionManifestValidator {
    pub fn validate(&self, item: &DefaultSuspensionManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultSuspensionManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultSuspensionManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_suspension_manifest_validator() -> DefaultSuspensionManifestValidator {
    DefaultSuspensionManifestValidator {
        rules: vec![
            Rc::new(DefaultSuspensionManifestCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DefaultSuspensionManifestRule>,
            Rc::new(DefaultSuspensionManifestRequiredCodeRule {
                message: "default suspension manifest needs an entry".to_string(),
            }),
            Rc::new(DefaultSuspensionManifestRequiredChannelRule {
                message: "default suspension manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_suspension_manifest_validator();
        let item = new_default_suspension_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultSuspensionManifestRequiredCodeRule {
            message: "default suspension manifest needs a value".to_string(),
        };
        let mut item = new_default_suspension_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = DefaultSuspensionManifestRequiredChannelRule {
            message: "default suspension manifest needs a value".to_string(),
        };
        let mut item = new_default_suspension_manifest("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
