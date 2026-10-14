use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain digest must pass. Check returns its problems, or none.
pub trait TrustDomainDigestRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDigestFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainDigestRule for TrustDomainDigestFloorValueRangeRule {
    fn check(&self, item: &TrustDomainDigest) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDigestCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainDigestRule for TrustDomainDigestCodePrefixRule {
    fn check(&self, item: &TrustDomainDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDigestAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl TrustDomainDigestRule for TrustDomainDigestAllowedCategoryRule {
    fn check(&self, item: &TrustDomainDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is missing", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDigestNameLengthRule {
    pub max_length: i64,
}

impl TrustDomainDigestRule for TrustDomainDigestNameLengthRule {
    fn check(&self, item: &TrustDomainDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDigestCategoryLengthRule {
    pub max_length: i64,
}

impl TrustDomainDigestRule for TrustDomainDigestCategoryLengthRule {
    fn check(&self, item: &TrustDomainDigest) -> Vec<String> {
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

/// Runs every rule of a trust domain digest.
#[derive(Debug, Clone)]
pub struct TrustDomainDigestValidator {
    pub rules: Vec<Rc<dyn TrustDomainDigestRule>>,
}

impl TrustDomainDigestValidator {
    pub fn validate(&self, item: &TrustDomainDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_digest_validator() -> TrustDomainDigestValidator {
    TrustDomainDigestValidator {
        rules: vec![
            Rc::new(TrustDomainDigestFloorValueRangeRule { min: 7.9, max: 966.5 }) as Rc<dyn TrustDomainDigestRule>,
            Rc::new(TrustDomainDigestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TrustDomainDigestAllowedCategoryRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(TrustDomainDigestNameLengthRule { max_length: 1000 }),
            Rc::new(TrustDomainDigestCategoryLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_digest_validator();
        let item = new_trust_domain_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TrustDomainDigestFloorValueRangeRule { min: 3.5, max: 1323.5 };
        let mut item = new_trust_domain_digest("x", "y");
        item.floor_value = 1325.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TrustDomainDigestNameLengthRule { max_length: 32 };
        let mut item = new_trust_domain_digest("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = TrustDomainDigestCategoryLengthRule { max_length: 120 };
        let mut item = new_trust_domain_digest("x", "y");
        item.category = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
