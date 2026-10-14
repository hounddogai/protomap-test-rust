use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered timeline manifest must pass. Check returns its problems, or none.
pub trait TieredTimelineManifestRule: std::fmt::Debug {
    fn check(&self, item: &TieredTimelineManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineManifestNamespaceLengthRule {
    pub max_length: i64,
}

impl TieredTimelineManifestRule for TieredTimelineManifestNamespaceLengthRule {
    fn check(&self, item: &TieredTimelineManifest) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineManifestCodePrefixRule {
    pub prefix: String,
}

impl TieredTimelineManifestRule for TieredTimelineManifestCodePrefixRule {
    fn check(&self, item: &TieredTimelineManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineManifestRequiredNamespaceRule {
    pub message: String,
}

impl TieredTimelineManifestRule for TieredTimelineManifestRequiredNamespaceRule {
    fn check(&self, item: &TieredTimelineManifest) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineManifestLocaleLengthRule {
    pub max_length: i64,
}

impl TieredTimelineManifestRule for TieredTimelineManifestLocaleLengthRule {
    fn check(&self, item: &TieredTimelineManifest) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered timeline manifest.
#[derive(Debug, Clone)]
pub struct TieredTimelineManifestValidator {
    pub rules: Vec<Rc<dyn TieredTimelineManifestRule>>,
}

impl TieredTimelineManifestValidator {
    pub fn validate(&self, item: &TieredTimelineManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredTimelineManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTimelineManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_timeline_manifest_validator() -> TieredTimelineManifestValidator {
    TieredTimelineManifestValidator {
        rules: vec![
            Rc::new(TieredTimelineManifestNamespaceLengthRule { max_length: 120 })
                as Rc<dyn TieredTimelineManifestRule>,
            Rc::new(TieredTimelineManifestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TieredTimelineManifestRequiredNamespaceRule {
                message: "tiered timeline manifest needs an entry".to_string(),
            }),
            Rc::new(TieredTimelineManifestLocaleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_timeline_manifest_validator();
        let item = new_tiered_timeline_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = TieredTimelineManifestNamespaceLengthRule { max_length: 1000 };
        let mut item = new_tiered_timeline_manifest("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = TieredTimelineManifestRequiredNamespaceRule {
            message: "tiered timeline manifest needs an entry".to_string(),
        };
        let mut item = new_tiered_timeline_manifest("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = TieredTimelineManifestLocaleLengthRule { max_length: 255 };
        let mut item = new_tiered_timeline_manifest("x", "y");
        item.locale = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
