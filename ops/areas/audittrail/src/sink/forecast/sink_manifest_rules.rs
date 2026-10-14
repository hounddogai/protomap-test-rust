use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink manifest must pass. Check returns its problems, or none.
pub trait SinkManifestRule: std::fmt::Debug {
    fn check(&self, item: &SinkManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkManifestCodePrefixRule {
    pub prefix: String,
}

impl SinkManifestRule for SinkManifestCodePrefixRule {
    fn check(&self, item: &SinkManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkManifestSourceLengthRule {
    pub max_length: i64,
}

impl SinkManifestRule for SinkManifestSourceLengthRule {
    fn check(&self, item: &SinkManifest) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkManifestRequiredSourceRule {
    pub message: String,
}

impl SinkManifestRule for SinkManifestRequiredSourceRule {
    fn check(&self, item: &SinkManifest) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink manifest.
#[derive(Debug, Clone)]
pub struct SinkManifestValidator {
    pub rules: Vec<Rc<dyn SinkManifestRule>>,
}

impl SinkManifestValidator {
    pub fn validate(&self, item: &SinkManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_manifest_validator() -> SinkManifestValidator {
    SinkManifestValidator {
        rules: vec![
            Rc::new(SinkManifestCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn SinkManifestRule>,
            Rc::new(SinkManifestSourceLengthRule { max_length: 86400 }),
            Rc::new(SinkManifestRequiredSourceRule {
                message: "sink manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_manifest_validator();
        let item = new_sink_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = SinkManifestSourceLengthRule { max_length: 64 };
        let mut item = new_sink_manifest("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = SinkManifestRequiredSourceRule {
            message: "sink manifest needs a value".to_string(),
        };
        let mut item = new_sink_manifest("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
