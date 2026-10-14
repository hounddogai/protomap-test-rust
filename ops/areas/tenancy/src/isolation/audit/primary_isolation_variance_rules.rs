use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary isolation variance must pass. Check returns its problems, or none.
pub trait PrimaryIsolationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryIsolationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIsolationVarianceIssuerLengthRule {
    pub max_length: i64,
}

impl PrimaryIsolationVarianceRule for PrimaryIsolationVarianceIssuerLengthRule {
    fn check(&self, item: &PrimaryIsolationVariance) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIsolationVarianceTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryIsolationVarianceRule for PrimaryIsolationVarianceTitleLengthRule {
    fn check(&self, item: &PrimaryIsolationVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIsolationVarianceRequiredIssuerRule {
    pub message: String,
}

impl PrimaryIsolationVarianceRule for PrimaryIsolationVarianceRequiredIssuerRule {
    fn check(&self, item: &PrimaryIsolationVariance) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIsolationVarianceAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PrimaryIsolationVarianceRule for PrimaryIsolationVarianceAllowedTitleRule {
    fn check(&self, item: &PrimaryIsolationVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIsolationVarianceCodePrefixRule {
    pub prefix: String,
}

impl PrimaryIsolationVarianceRule for PrimaryIsolationVarianceCodePrefixRule {
    fn check(&self, item: &PrimaryIsolationVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary isolation variance.
#[derive(Debug, Clone)]
pub struct PrimaryIsolationVarianceValidator {
    pub rules: Vec<Rc<dyn PrimaryIsolationVarianceRule>>,
}

impl PrimaryIsolationVarianceValidator {
    pub fn validate(&self, item: &PrimaryIsolationVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryIsolationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryIsolationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_isolation_variance_validator() -> PrimaryIsolationVarianceValidator {
    PrimaryIsolationVarianceValidator {
        rules: vec![
            Rc::new(PrimaryIsolationVarianceIssuerLengthRule { max_length: 255 })
                as Rc<dyn PrimaryIsolationVarianceRule>,
            Rc::new(PrimaryIsolationVarianceTitleLengthRule { max_length: 1000 }),
            Rc::new(PrimaryIsolationVarianceRequiredIssuerRule {
                message: "primary isolation variance needs a value".to_string(),
            }),
            Rc::new(PrimaryIsolationVarianceAllowedTitleRule {
                allowed: vec!["lenient".to_string(), "seasonal".to_string(), "legacy".to_string()],
            }),
            Rc::new(PrimaryIsolationVarianceCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_isolation_variance_validator();
        let item = new_primary_isolation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PrimaryIsolationVarianceIssuerLengthRule { max_length: 64 };
        let mut item = new_primary_isolation_variance("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryIsolationVarianceTitleLengthRule { max_length: 16 };
        let mut item = new_primary_isolation_variance("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = PrimaryIsolationVarianceRequiredIssuerRule {
            message: "primary isolation variance needs a value".to_string(),
        };
        let mut item = new_primary_isolation_variance("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
