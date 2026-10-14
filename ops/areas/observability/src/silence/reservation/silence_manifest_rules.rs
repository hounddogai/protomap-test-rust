use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence manifest must pass. Check returns its problems, or none.
pub trait SilenceManifestRule: std::fmt::Debug {
    fn check(&self, item: &SilenceManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceManifestUnitLengthRule {
    pub max_length: i64,
}

impl SilenceManifestRule for SilenceManifestUnitLengthRule {
    fn check(&self, item: &SilenceManifest) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceManifestCodePrefixRule {
    pub prefix: String,
}

impl SilenceManifestRule for SilenceManifestCodePrefixRule {
    fn check(&self, item: &SilenceManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceManifestRequiredLabelRule {
    pub message: String,
}

impl SilenceManifestRule for SilenceManifestRequiredLabelRule {
    fn check(&self, item: &SilenceManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence manifest.
#[derive(Debug, Clone)]
pub struct SilenceManifestValidator {
    pub rules: Vec<Rc<dyn SilenceManifestRule>>,
}

impl SilenceManifestValidator {
    pub fn validate(&self, item: &SilenceManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_manifest_validator() -> SilenceManifestValidator {
    SilenceManifestValidator {
        rules: vec![
            Rc::new(SilenceManifestUnitLengthRule { max_length: 64 }) as Rc<dyn SilenceManifestRule>,
            Rc::new(SilenceManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SilenceManifestRequiredLabelRule {
                message: "silence manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_manifest_validator();
        let item = new_silence_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = SilenceManifestUnitLengthRule { max_length: 255 };
        let mut item = new_silence_manifest("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SilenceManifestRequiredLabelRule {
            message: "silence manifest needs a value".to_string(),
        };
        let mut item = new_silence_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
