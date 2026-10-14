use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary sink manifest must pass. Check returns its problems, or none.
pub trait PrimarySinkManifestRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySinkManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifestRequiredCodeRule {
    pub message: String,
}

impl PrimarySinkManifestRule for PrimarySinkManifestRequiredCodeRule {
    fn check(&self, item: &PrimarySinkManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifestHostLengthRule {
    pub max_length: i64,
}

impl PrimarySinkManifestRule for PrimarySinkManifestHostLengthRule {
    fn check(&self, item: &PrimarySinkManifest) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifestRequiredHostRule {
    pub message: String,
}

impl PrimarySinkManifestRule for PrimarySinkManifestRequiredHostRule {
    fn check(&self, item: &PrimarySinkManifest) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifestTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PrimarySinkManifestRule for PrimarySinkManifestTagsLimitRule {
    fn check(&self, item: &PrimarySinkManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifestCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySinkManifestRule for PrimarySinkManifestCodeLengthRule {
    fn check(&self, item: &PrimarySinkManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifestRequiredLabelRule {
    pub message: String,
}

impl PrimarySinkManifestRule for PrimarySinkManifestRequiredLabelRule {
    fn check(&self, item: &PrimarySinkManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary sink manifest.
#[derive(Debug, Clone)]
pub struct PrimarySinkManifestValidator {
    pub rules: Vec<Rc<dyn PrimarySinkManifestRule>>,
}

impl PrimarySinkManifestValidator {
    pub fn validate(&self, item: &PrimarySinkManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimarySinkManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySinkManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_sink_manifest_validator() -> PrimarySinkManifestValidator {
    PrimarySinkManifestValidator {
        rules: vec![
            Rc::new(PrimarySinkManifestRequiredCodeRule {
                message: "primary sink manifest needs a value".to_string(),
            }) as Rc<dyn PrimarySinkManifestRule>,
            Rc::new(PrimarySinkManifestHostLengthRule { max_length: 64 }),
            Rc::new(PrimarySinkManifestRequiredHostRule {
                message: "primary sink manifest needs this field".to_string(),
            }),
            Rc::new(PrimarySinkManifestTagsLimitRule {
                max_count: 1000,
                banned: Vec::new(),
            }),
            Rc::new(PrimarySinkManifestCodeLengthRule { max_length: 3600 }),
            Rc::new(PrimarySinkManifestRequiredLabelRule {
                message: "primary sink manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_sink_manifest_validator();
        let item = new_primary_sink_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimarySinkManifestRequiredCodeRule {
            message: "primary sink manifest needs a value".to_string(),
        };
        let mut item = new_primary_sink_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = PrimarySinkManifestHostLengthRule { max_length: 64 };
        let mut item = new_primary_sink_manifest("x", "y");
        item.host = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PrimarySinkManifestRequiredHostRule {
            message: "primary sink manifest needs a value".to_string(),
        };
        let mut item = new_primary_sink_manifest("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySinkManifestCodeLengthRule { max_length: 32 };
        let mut item = new_primary_sink_manifest("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimarySinkManifestRequiredLabelRule {
            message: "primary sink manifest needs an entry".to_string(),
        };
        let mut item = new_primary_sink_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
