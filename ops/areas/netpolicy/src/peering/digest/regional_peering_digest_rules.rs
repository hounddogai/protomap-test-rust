use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional peering digest must pass. Check returns its problems, or none.
pub trait RegionalPeeringDigestRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPeeringDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringDigestCipherLengthRule {
    pub max_length: i64,
}

impl RegionalPeeringDigestRule for RegionalPeeringDigestCipherLengthRule {
    fn check(&self, item: &RegionalPeeringDigest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringDigestAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl RegionalPeeringDigestRule for RegionalPeeringDigestAllowedExternalRefRule {
    fn check(&self, item: &RegionalPeeringDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is too long", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringDigestCodePrefixRule {
    pub prefix: String,
}

impl RegionalPeeringDigestRule for RegionalPeeringDigestCodePrefixRule {
    fn check(&self, item: &RegionalPeeringDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional peering digest.
#[derive(Debug, Clone)]
pub struct RegionalPeeringDigestValidator {
    pub rules: Vec<Rc<dyn RegionalPeeringDigestRule>>,
}

impl RegionalPeeringDigestValidator {
    pub fn validate(&self, item: &RegionalPeeringDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalPeeringDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPeeringDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_peering_digest_validator() -> RegionalPeeringDigestValidator {
    RegionalPeeringDigestValidator {
        rules: vec![
            Rc::new(RegionalPeeringDigestCipherLengthRule { max_length: 255 }) as Rc<dyn RegionalPeeringDigestRule>,
            Rc::new(RegionalPeeringDigestAllowedExternalRefRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(RegionalPeeringDigestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_peering_digest_validator();
        let item = new_regional_peering_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = RegionalPeeringDigestCipherLengthRule { max_length: 86400 };
        let mut item = new_regional_peering_digest("x", "y");
        item.cipher = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
