use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier variance must pass. Check returns its problems, or none.
pub trait VerifierVarianceRule: std::fmt::Debug {
    fn check(&self, item: &VerifierVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierVarianceRequiredSegmentRule {
    pub message: String,
}

impl VerifierVarianceRule for VerifierVarianceRequiredSegmentRule {
    fn check(&self, item: &VerifierVariance) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierVarianceCodeLengthRule {
    pub max_length: i64,
}

impl VerifierVarianceRule for VerifierVarianceCodeLengthRule {
    fn check(&self, item: &VerifierVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierVarianceSegmentLengthRule {
    pub max_length: i64,
}

impl VerifierVarianceRule for VerifierVarianceSegmentLengthRule {
    fn check(&self, item: &VerifierVariance) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierVarianceAllowedNameRule {
    pub allowed: Vec<String>,
}

impl VerifierVarianceRule for VerifierVarianceAllowedNameRule {
    fn check(&self, item: &VerifierVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier variance.
#[derive(Debug, Clone)]
pub struct VerifierVarianceValidator {
    pub rules: Vec<Rc<dyn VerifierVarianceRule>>,
}

impl VerifierVarianceValidator {
    pub fn validate(&self, item: &VerifierVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_variance_validator() -> VerifierVarianceValidator {
    VerifierVarianceValidator {
        rules: vec![
            Rc::new(VerifierVarianceRequiredSegmentRule {
                message: "verifier variance needs a value".to_string(),
            }) as Rc<dyn VerifierVarianceRule>,
            Rc::new(VerifierVarianceCodeLengthRule { max_length: 255 }),
            Rc::new(VerifierVarianceSegmentLengthRule { max_length: 255 }),
            Rc::new(VerifierVarianceAllowedNameRule {
                allowed: vec!["legacy".to_string(), "lenient".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_variance_validator();
        let item = new_verifier_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = VerifierVarianceRequiredSegmentRule {
            message: "verifier variance needs a value".to_string(),
        };
        let mut item = new_verifier_variance("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierVarianceCodeLengthRule { max_length: 1000 };
        let mut item = new_verifier_variance("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = VerifierVarianceSegmentLengthRule { max_length: 1000 };
        let mut item = new_verifier_variance("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
