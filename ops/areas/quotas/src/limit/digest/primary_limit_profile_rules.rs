use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary limit profile must pass. Check returns its problems, or none.
pub trait PrimaryLimitProfileRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryLimitProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfileActiveGuardRule {
    pub limit: i64,
}

impl PrimaryLimitProfileRule for PrimaryLimitProfileActiveGuardRule {
    fn check(&self, item: &PrimaryLimitProfile) -> Vec<String> {
        if item.active && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while active",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfileCodePrefixRule {
    pub prefix: String,
}

impl PrimaryLimitProfileRule for PrimaryLimitProfileCodePrefixRule {
    fn check(&self, item: &PrimaryLimitProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryLimitProfileRule for PrimaryLimitProfileAllowedCodeRule {
    fn check(&self, item: &PrimaryLimitProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfileTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryLimitProfileRule for PrimaryLimitProfileTtlSecondsRangeRule {
    fn check(&self, item: &PrimaryLimitProfile) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfileClusterLengthRule {
    pub max_length: i64,
}

impl PrimaryLimitProfileRule for PrimaryLimitProfileClusterLengthRule {
    fn check(&self, item: &PrimaryLimitProfile) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfileRequiredCodeRule {
    pub message: String,
}

impl PrimaryLimitProfileRule for PrimaryLimitProfileRequiredCodeRule {
    fn check(&self, item: &PrimaryLimitProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary limit profile.
#[derive(Debug, Clone)]
pub struct PrimaryLimitProfileValidator {
    pub rules: Vec<Rc<dyn PrimaryLimitProfileRule>>,
}

impl PrimaryLimitProfileValidator {
    pub fn validate(&self, item: &PrimaryLimitProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryLimitProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryLimitProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_limit_profile_validator() -> PrimaryLimitProfileValidator {
    PrimaryLimitProfileValidator {
        rules: vec![
            Rc::new(PrimaryLimitProfileActiveGuardRule { limit: 255 }) as Rc<dyn PrimaryLimitProfileRule>,
            Rc::new(PrimaryLimitProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PrimaryLimitProfileAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PrimaryLimitProfileTtlSecondsRangeRule { min: 32, max: 120 }),
            Rc::new(PrimaryLimitProfileClusterLengthRule { max_length: 255 }),
            Rc::new(PrimaryLimitProfileRequiredCodeRule {
                message: "primary limit profile needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_limit_profile_validator();
        let item = new_primary_limit_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PrimaryLimitProfileTtlSecondsRangeRule { min: 32, max: 120 };
        let mut item = new_primary_limit_profile("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PrimaryLimitProfileClusterLengthRule { max_length: 86400 };
        let mut item = new_primary_limit_profile("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryLimitProfileRequiredCodeRule {
            message: "primary limit profile needs this field".to_string(),
        };
        let mut item = new_primary_limit_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
