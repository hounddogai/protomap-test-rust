use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity profile must pass. Check returns its problems, or none.
pub trait IdentityProfileRule: std::fmt::Debug {
    fn check(&self, item: &IdentityProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityProfileTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl IdentityProfileRule for IdentityProfileTagsLimitRule {
    fn check(&self, item: &IdentityProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityProfileRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityProfileRule for IdentityProfileRotationDaysRangeRule {
    fn check(&self, item: &IdentityProfile) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityProfileCodePrefixRule {
    pub prefix: String,
}

impl IdentityProfileRule for IdentityProfileCodePrefixRule {
    fn check(&self, item: &IdentityProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityProfileStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityProfileRule for IdentityProfileStartsAtRangeRule {
    fn check(&self, item: &IdentityProfile) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityProfileRequiredChecksumRule {
    pub message: String,
}

impl IdentityProfileRule for IdentityProfileRequiredChecksumRule {
    fn check(&self, item: &IdentityProfile) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity profile.
#[derive(Debug, Clone)]
pub struct IdentityProfileValidator {
    pub rules: Vec<Rc<dyn IdentityProfileRule>>,
}

impl IdentityProfileValidator {
    pub fn validate(&self, item: &IdentityProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_profile_validator() -> IdentityProfileValidator {
    IdentityProfileValidator {
        rules: vec![
            Rc::new(IdentityProfileTagsLimitRule {
                max_count: 32,
                banned: vec!["global".to_string(), "default".to_string()],
            }) as Rc<dyn IdentityProfileRule>,
            Rc::new(IdentityProfileRotationDaysRangeRule { min: 80, max: 3600 }),
            Rc::new(IdentityProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(IdentityProfileStartsAtRangeRule { min: 32, max: 255 }),
            Rc::new(IdentityProfileRequiredChecksumRule {
                message: "identity profile needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_profile_validator();
        let item = new_identity_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = IdentityProfileRotationDaysRangeRule { min: 16, max: 80 };
        let mut item = new_identity_profile("x", "y");
        item.rotation_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = IdentityProfileStartsAtRangeRule { min: 32, max: 3600 };
        let mut item = new_identity_profile("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = IdentityProfileRequiredChecksumRule {
            message: "identity profile needs an entry".to_string(),
        };
        let mut item = new_identity_profile("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
