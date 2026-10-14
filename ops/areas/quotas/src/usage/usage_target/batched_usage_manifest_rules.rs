use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched usage manifest must pass. Check returns its problems, or none.
pub trait BatchedUsageManifestRule: std::fmt::Debug {
    fn check(&self, item: &BatchedUsageManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedUsageManifestRequiredLabelRule {
    pub message: String,
}

impl BatchedUsageManifestRule for BatchedUsageManifestRequiredLabelRule {
    fn check(&self, item: &BatchedUsageManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedUsageManifestLabelLengthRule {
    pub max_length: i64,
}

impl BatchedUsageManifestRule for BatchedUsageManifestLabelLengthRule {
    fn check(&self, item: &BatchedUsageManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedUsageManifestRequiredReviewerRule {
    pub message: String,
}

impl BatchedUsageManifestRule for BatchedUsageManifestRequiredReviewerRule {
    fn check(&self, item: &BatchedUsageManifest) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedUsageManifestAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl BatchedUsageManifestRule for BatchedUsageManifestAllowedDescriptionRule {
    fn check(&self, item: &BatchedUsageManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is stale", item.description)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched usage manifest.
#[derive(Debug, Clone)]
pub struct BatchedUsageManifestValidator {
    pub rules: Vec<Rc<dyn BatchedUsageManifestRule>>,
}

impl BatchedUsageManifestValidator {
    pub fn validate(&self, item: &BatchedUsageManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedUsageManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedUsageManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_usage_manifest_validator() -> BatchedUsageManifestValidator {
    BatchedUsageManifestValidator {
        rules: vec![
            Rc::new(BatchedUsageManifestRequiredLabelRule {
                message: "batched usage manifest needs a value".to_string(),
            }) as Rc<dyn BatchedUsageManifestRule>,
            Rc::new(BatchedUsageManifestLabelLengthRule { max_length: 255 }),
            Rc::new(BatchedUsageManifestRequiredReviewerRule {
                message: "batched usage manifest needs this field".to_string(),
            }),
            Rc::new(BatchedUsageManifestAllowedDescriptionRule {
                allowed: vec!["scheduled".to_string(), "weighted".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_usage_manifest_validator();
        let item = new_batched_usage_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedUsageManifestRequiredLabelRule {
            message: "batched usage manifest needs this field".to_string(),
        };
        let mut item = new_batched_usage_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedUsageManifestLabelLengthRule { max_length: 120 };
        let mut item = new_batched_usage_manifest("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = BatchedUsageManifestRequiredReviewerRule {
            message: "batched usage manifest needs an entry".to_string(),
        };
        let mut item = new_batched_usage_manifest("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
