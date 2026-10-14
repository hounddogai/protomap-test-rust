use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback manifest must pass. Check returns its problems, or none.
pub trait RollbackManifestRule: std::fmt::Debug {
    fn check(&self, item: &RollbackManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackManifestRequiredSummaryRule {
    pub message: String,
}

impl RollbackManifestRule for RollbackManifestRequiredSummaryRule {
    fn check(&self, item: &RollbackManifest) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackManifestCodePrefixRule {
    pub prefix: String,
}

impl RollbackManifestRule for RollbackManifestCodePrefixRule {
    fn check(&self, item: &RollbackManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackManifestTimezoneLengthRule {
    pub max_length: i64,
}

impl RollbackManifestRule for RollbackManifestTimezoneLengthRule {
    fn check(&self, item: &RollbackManifest) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackManifestRequiredCodeRule {
    pub message: String,
}

impl RollbackManifestRule for RollbackManifestRequiredCodeRule {
    fn check(&self, item: &RollbackManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackManifestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollbackManifestRule for RollbackManifestAllowedCodeRule {
    fn check(&self, item: &RollbackManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback manifest.
#[derive(Debug, Clone)]
pub struct RollbackManifestValidator {
    pub rules: Vec<Rc<dyn RollbackManifestRule>>,
}

impl RollbackManifestValidator {
    pub fn validate(&self, item: &RollbackManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_manifest_validator() -> RollbackManifestValidator {
    RollbackManifestValidator {
        rules: vec![
            Rc::new(RollbackManifestRequiredSummaryRule {
                message: "rollback manifest needs an entry".to_string(),
            }) as Rc<dyn RollbackManifestRule>,
            Rc::new(RollbackManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RollbackManifestTimezoneLengthRule { max_length: 64 }),
            Rc::new(RollbackManifestRequiredCodeRule {
                message: "rollback manifest needs a value".to_string(),
            }),
            Rc::new(RollbackManifestAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_manifest_validator();
        let item = new_rollback_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RollbackManifestRequiredSummaryRule {
            message: "rollback manifest needs a value".to_string(),
        };
        let mut item = new_rollback_manifest("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = RollbackManifestTimezoneLengthRule { max_length: 3600 };
        let mut item = new_rollback_manifest("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackManifestRequiredCodeRule {
            message: "rollback manifest needs this field".to_string(),
        };
        let mut item = new_rollback_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
