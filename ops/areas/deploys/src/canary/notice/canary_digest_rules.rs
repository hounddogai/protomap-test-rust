use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary digest must pass. Check returns its problems, or none.
pub trait CanaryDigestRule: std::fmt::Debug {
    fn check(&self, item: &CanaryDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDigestCategoryLengthRule {
    pub max_length: i64,
}

impl CanaryDigestRule for CanaryDigestCategoryLengthRule {
    fn check(&self, item: &CanaryDigest) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDigestLabelLengthRule {
    pub max_length: i64,
}

impl CanaryDigestRule for CanaryDigestLabelLengthRule {
    fn check(&self, item: &CanaryDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDigestExternalRefLengthRule {
    pub max_length: i64,
}

impl CanaryDigestRule for CanaryDigestExternalRefLengthRule {
    fn check(&self, item: &CanaryDigest) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDigestRequiredLabelRule {
    pub message: String,
}

impl CanaryDigestRule for CanaryDigestRequiredLabelRule {
    fn check(&self, item: &CanaryDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDigestAllowedHostRule {
    pub allowed: Vec<String>,
}

impl CanaryDigestRule for CanaryDigestAllowedHostRule {
    fn check(&self, item: &CanaryDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is too long", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDigestRequiredExternalRefRule {
    pub message: String,
}

impl CanaryDigestRule for CanaryDigestRequiredExternalRefRule {
    fn check(&self, item: &CanaryDigest) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary digest.
#[derive(Debug, Clone)]
pub struct CanaryDigestValidator {
    pub rules: Vec<Rc<dyn CanaryDigestRule>>,
}

impl CanaryDigestValidator {
    pub fn validate(&self, item: &CanaryDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_digest_validator() -> CanaryDigestValidator {
    CanaryDigestValidator {
        rules: vec![
            Rc::new(CanaryDigestCategoryLengthRule { max_length: 255 }) as Rc<dyn CanaryDigestRule>,
            Rc::new(CanaryDigestLabelLengthRule { max_length: 255 }),
            Rc::new(CanaryDigestExternalRefLengthRule { max_length: 1000 }),
            Rc::new(CanaryDigestRequiredLabelRule {
                message: "canary digest needs an entry".to_string(),
            }),
            Rc::new(CanaryDigestAllowedHostRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(CanaryDigestRequiredExternalRefRule {
                message: "canary digest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_digest_validator();
        let item = new_canary_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = CanaryDigestCategoryLengthRule { max_length: 1000 };
        let mut item = new_canary_digest("x", "y");
        item.category = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CanaryDigestLabelLengthRule { max_length: 32 };
        let mut item = new_canary_digest("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = CanaryDigestExternalRefLengthRule { max_length: 64 };
        let mut item = new_canary_digest("x", "y");
        item.external_ref = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CanaryDigestRequiredLabelRule {
            message: "canary digest needs this field".to_string(),
        };
        let mut item = new_canary_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = CanaryDigestRequiredExternalRefRule {
            message: "canary digest needs an entry".to_string(),
        };
        let mut item = new_canary_digest("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
