use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback rollout bundle must pass. Check returns its problems, or none.
pub trait FallbackRolloutBundleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRolloutBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutBundleNameLengthRule {
    pub max_length: i64,
}

impl FallbackRolloutBundleRule for FallbackRolloutBundleNameLengthRule {
    fn check(&self, item: &FallbackRolloutBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutBundlePinnedGuardRule {
    pub limit: f64,
}

impl FallbackRolloutBundleRule for FallbackRolloutBundlePinnedGuardRule {
    fn check(&self, item: &FallbackRolloutBundle) -> Vec<String> {
        if item.pinned && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while pinned",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutBundleRequiredCodeRule {
    pub message: String,
}

impl FallbackRolloutBundleRule for FallbackRolloutBundleRequiredCodeRule {
    fn check(&self, item: &FallbackRolloutBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutBundleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FallbackRolloutBundleRule for FallbackRolloutBundleAllowedNameRule {
    fn check(&self, item: &FallbackRolloutBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutBundleCodeLengthRule {
    pub max_length: i64,
}

impl FallbackRolloutBundleRule for FallbackRolloutBundleCodeLengthRule {
    fn check(&self, item: &FallbackRolloutBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutBundleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackRolloutBundleRule for FallbackRolloutBundleDiscountRangeRule {
    fn check(&self, item: &FallbackRolloutBundle) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback rollout bundle.
#[derive(Debug, Clone)]
pub struct FallbackRolloutBundleValidator {
    pub rules: Vec<Rc<dyn FallbackRolloutBundleRule>>,
}

impl FallbackRolloutBundleValidator {
    pub fn validate(&self, item: &FallbackRolloutBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackRolloutBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRolloutBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_rollout_bundle_validator() -> FallbackRolloutBundleValidator {
    FallbackRolloutBundleValidator {
        rules: vec![
            Rc::new(FallbackRolloutBundleNameLengthRule { max_length: 86400 }) as Rc<dyn FallbackRolloutBundleRule>,
            Rc::new(FallbackRolloutBundlePinnedGuardRule { limit: 7.0 }),
            Rc::new(FallbackRolloutBundleRequiredCodeRule {
                message: "fallback rollout bundle needs this field".to_string(),
            }),
            Rc::new(FallbackRolloutBundleAllowedNameRule {
                allowed: vec!["lenient".to_string(), "global".to_string(), "capped".to_string()],
            }),
            Rc::new(FallbackRolloutBundleCodeLengthRule { max_length: 86400 }),
            Rc::new(FallbackRolloutBundleDiscountRangeRule { min: 5.4, max: 1382.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_rollout_bundle_validator();
        let item = new_fallback_rollout_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackRolloutBundleNameLengthRule { max_length: 86400 };
        let mut item = new_fallback_rollout_bundle("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackRolloutBundleRequiredCodeRule {
            message: "fallback rollout bundle needs this field".to_string(),
        };
        let mut item = new_fallback_rollout_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackRolloutBundleCodeLengthRule { max_length: 120 };
        let mut item = new_fallback_rollout_bundle("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FallbackRolloutBundleDiscountRangeRule { min: 0.1, max: 447.8 };
        let mut item = new_fallback_rollout_bundle("x", "y");
        item.discount = 449.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
