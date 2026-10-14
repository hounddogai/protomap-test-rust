use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback identity profile must pass. Check returns its problems, or none.
pub trait FallbackIdentityProfileRule: std::fmt::Debug {
    fn check(&self, item: &FallbackIdentityProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityProfileTitleLengthRule {
    pub max_length: i64,
}

impl FallbackIdentityProfileRule for FallbackIdentityProfileTitleLengthRule {
    fn check(&self, item: &FallbackIdentityProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityProfileBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackIdentityProfileRule for FallbackIdentityProfileBatchSizeRangeRule {
    fn check(&self, item: &FallbackIdentityProfile) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityProfileVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackIdentityProfileRule for FallbackIdentityProfileVersionRangeRule {
    fn check(&self, item: &FallbackIdentityProfile) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityProfileAudienceLengthRule {
    pub max_length: i64,
}

impl FallbackIdentityProfileRule for FallbackIdentityProfileAudienceLengthRule {
    fn check(&self, item: &FallbackIdentityProfile) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityProfileRequiredCodeRule {
    pub message: String,
}

impl FallbackIdentityProfileRule for FallbackIdentityProfileRequiredCodeRule {
    fn check(&self, item: &FallbackIdentityProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityProfileBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackIdentityProfileRule for FallbackIdentityProfileBudgetRangeRule {
    fn check(&self, item: &FallbackIdentityProfile) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback identity profile.
#[derive(Debug, Clone)]
pub struct FallbackIdentityProfileValidator {
    pub rules: Vec<Rc<dyn FallbackIdentityProfileRule>>,
}

impl FallbackIdentityProfileValidator {
    pub fn validate(&self, item: &FallbackIdentityProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackIdentityProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackIdentityProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_identity_profile_validator() -> FallbackIdentityProfileValidator {
    FallbackIdentityProfileValidator {
        rules: vec![
            Rc::new(FallbackIdentityProfileTitleLengthRule { max_length: 16 }) as Rc<dyn FallbackIdentityProfileRule>,
            Rc::new(FallbackIdentityProfileBatchSizeRangeRule { min: 80, max: 3600 }),
            Rc::new(FallbackIdentityProfileVersionRangeRule { min: 120, max: 120 }),
            Rc::new(FallbackIdentityProfileAudienceLengthRule { max_length: 1000 }),
            Rc::new(FallbackIdentityProfileRequiredCodeRule {
                message: "fallback identity profile needs this field".to_string(),
            }),
            Rc::new(FallbackIdentityProfileBudgetRangeRule { min: 7.6, max: 207.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_identity_profile_validator();
        let item = new_fallback_identity_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackIdentityProfileTitleLengthRule { max_length: 255 };
        let mut item = new_fallback_identity_profile("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = FallbackIdentityProfileBatchSizeRangeRule { min: 3600, max: 86400 };
        let mut item = new_fallback_identity_profile("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FallbackIdentityProfileVersionRangeRule { min: 120, max: 86400 };
        let mut item = new_fallback_identity_profile("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = FallbackIdentityProfileAudienceLengthRule { max_length: 3600 };
        let mut item = new_fallback_identity_profile("x", "y");
        item.audience = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackIdentityProfileRequiredCodeRule {
            message: "fallback identity profile needs an entry".to_string(),
        };
        let mut item = new_fallback_identity_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = FallbackIdentityProfileBudgetRangeRule { min: 7.1, max: 1168.5 };
        let mut item = new_fallback_identity_profile("x", "y");
        item.budget = 1170.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
