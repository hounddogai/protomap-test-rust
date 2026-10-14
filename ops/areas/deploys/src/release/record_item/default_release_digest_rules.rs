use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default release digest must pass. Check returns its problems, or none.
pub trait DefaultReleaseDigestRule: std::fmt::Debug {
    fn check(&self, item: &DefaultReleaseDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseDigestAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl DefaultReleaseDigestRule for DefaultReleaseDigestAllowedRegionRule {
    fn check(&self, item: &DefaultReleaseDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is below minimum", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseDigestRequiredRegionRule {
    pub message: String,
}

impl DefaultReleaseDigestRule for DefaultReleaseDigestRequiredRegionRule {
    fn check(&self, item: &DefaultReleaseDigest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseDigestRegionLengthRule {
    pub max_length: i64,
}

impl DefaultReleaseDigestRule for DefaultReleaseDigestRegionLengthRule {
    fn check(&self, item: &DefaultReleaseDigest) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseDigestCodePrefixRule {
    pub prefix: String,
}

impl DefaultReleaseDigestRule for DefaultReleaseDigestCodePrefixRule {
    fn check(&self, item: &DefaultReleaseDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default release digest.
#[derive(Debug, Clone)]
pub struct DefaultReleaseDigestValidator {
    pub rules: Vec<Rc<dyn DefaultReleaseDigestRule>>,
}

impl DefaultReleaseDigestValidator {
    pub fn validate(&self, item: &DefaultReleaseDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultReleaseDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultReleaseDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_release_digest_validator() -> DefaultReleaseDigestValidator {
    DefaultReleaseDigestValidator {
        rules: vec![
            Rc::new(DefaultReleaseDigestAllowedRegionRule {
                allowed: vec!["manual".to_string(), "default".to_string(), "capped".to_string()],
            }) as Rc<dyn DefaultReleaseDigestRule>,
            Rc::new(DefaultReleaseDigestRequiredRegionRule {
                message: "default release digest needs an entry".to_string(),
            }),
            Rc::new(DefaultReleaseDigestRegionLengthRule { max_length: 120 }),
            Rc::new(DefaultReleaseDigestCodePrefixRule {
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
        let validator = default_default_release_digest_validator();
        let item = new_default_release_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DefaultReleaseDigestRequiredRegionRule {
            message: "default release digest needs an entry".to_string(),
        };
        let mut item = new_default_release_digest("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = DefaultReleaseDigestRegionLengthRule { max_length: 86400 };
        let mut item = new_default_release_digest("x", "y");
        item.region = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
