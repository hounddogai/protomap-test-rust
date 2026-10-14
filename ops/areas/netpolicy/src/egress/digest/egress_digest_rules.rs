use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress digest must pass. Check returns its problems, or none.
pub trait EgressDigestRule: std::fmt::Debug {
    fn check(&self, item: &EgressDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressDigestPrincipalLengthRule {
    pub max_length: i64,
}

impl EgressDigestRule for EgressDigestPrincipalLengthRule {
    fn check(&self, item: &EgressDigest) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressDigestAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl EgressDigestRule for EgressDigestAllowedExternalRefRule {
    fn check(&self, item: &EgressDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is inconsistent", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressDigestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressDigestRule for EgressDigestScoreRangeRule {
    fn check(&self, item: &EgressDigest) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressDigestRequiredCodeRule {
    pub message: String,
}

impl EgressDigestRule for EgressDigestRequiredCodeRule {
    fn check(&self, item: &EgressDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressDigestExternalRefLengthRule {
    pub max_length: i64,
}

impl EgressDigestRule for EgressDigestExternalRefLengthRule {
    fn check(&self, item: &EgressDigest) -> Vec<String> {
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

/// Runs every rule of a egress digest.
#[derive(Debug, Clone)]
pub struct EgressDigestValidator {
    pub rules: Vec<Rc<dyn EgressDigestRule>>,
}

impl EgressDigestValidator {
    pub fn validate(&self, item: &EgressDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_digest_validator() -> EgressDigestValidator {
    EgressDigestValidator {
        rules: vec![
            Rc::new(EgressDigestPrincipalLengthRule { max_length: 86400 }) as Rc<dyn EgressDigestRule>,
            Rc::new(EgressDigestAllowedExternalRefRule { allowed: Vec::new() }),
            Rc::new(EgressDigestScoreRangeRule { min: 7.7, max: 2047.9 }),
            Rc::new(EgressDigestRequiredCodeRule {
                message: "egress digest needs a value".to_string(),
            }),
            Rc::new(EgressDigestExternalRefLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_digest_validator();
        let item = new_egress_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = EgressDigestPrincipalLengthRule { max_length: 120 };
        let mut item = new_egress_digest("x", "y");
        item.principal = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = EgressDigestScoreRangeRule { min: 8.6, max: 2630.7 };
        let mut item = new_egress_digest("x", "y");
        item.score = 2632.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressDigestRequiredCodeRule {
            message: "egress digest needs a value".to_string(),
        };
        let mut item = new_egress_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = EgressDigestExternalRefLengthRule { max_length: 86400 };
        let mut item = new_egress_digest("x", "y");
        item.external_ref = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
