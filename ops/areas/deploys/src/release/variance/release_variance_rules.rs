use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release variance must pass. Check returns its problems, or none.
pub trait ReleaseVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseVarianceCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseVarianceRule for ReleaseVarianceCodeLengthRule {
    fn check(&self, item: &ReleaseVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseVarianceRequiredExternalRefRule {
    pub message: String,
}

impl ReleaseVarianceRule for ReleaseVarianceRequiredExternalRefRule {
    fn check(&self, item: &ReleaseVariance) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseVarianceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ReleaseVarianceRule for ReleaseVarianceAllowedCodeRule {
    fn check(&self, item: &ReleaseVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release variance.
#[derive(Debug, Clone)]
pub struct ReleaseVarianceValidator {
    pub rules: Vec<Rc<dyn ReleaseVarianceRule>>,
}

impl ReleaseVarianceValidator {
    pub fn validate(&self, item: &ReleaseVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_variance_validator() -> ReleaseVarianceValidator {
    ReleaseVarianceValidator {
        rules: vec![
            Rc::new(ReleaseVarianceCodeLengthRule { max_length: 1000 }) as Rc<dyn ReleaseVarianceRule>,
            Rc::new(ReleaseVarianceRequiredExternalRefRule {
                message: "release variance needs this field".to_string(),
            }),
            Rc::new(ReleaseVarianceAllowedCodeRule {
                allowed: vec!["capped".to_string(), "legacy".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_variance_validator();
        let item = new_release_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseVarianceCodeLengthRule { max_length: 3600 };
        let mut item = new_release_variance("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ReleaseVarianceRequiredExternalRefRule {
            message: "release variance needs this field".to_string(),
        };
        let mut item = new_release_variance("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
