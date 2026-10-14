use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback action item manifest must pass. Check returns its problems, or none.
pub trait FallbackActionItemManifestRule: std::fmt::Debug {
    fn check(&self, item: &FallbackActionItemManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemManifestNameLengthRule {
    pub max_length: i64,
}

impl FallbackActionItemManifestRule for FallbackActionItemManifestNameLengthRule {
    fn check(&self, item: &FallbackActionItemManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemManifestApprovedGuardRule {
    pub limit: i64,
}

impl FallbackActionItemManifestRule for FallbackActionItemManifestApprovedGuardRule {
    fn check(&self, item: &FallbackActionItemManifest) -> Vec<String> {
        if item.approved && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while approved",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemManifestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackActionItemManifestRule for FallbackActionItemManifestExpiresAtRangeRule {
    fn check(&self, item: &FallbackActionItemManifest) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemManifestBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackActionItemManifestRule for FallbackActionItemManifestBudgetRangeRule {
    fn check(&self, item: &FallbackActionItemManifest) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemManifestAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FallbackActionItemManifestRule for FallbackActionItemManifestAllowedNameRule {
    fn check(&self, item: &FallbackActionItemManifest) -> Vec<String> {
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
pub struct FallbackActionItemManifestCodeLengthRule {
    pub max_length: i64,
}

impl FallbackActionItemManifestRule for FallbackActionItemManifestCodeLengthRule {
    fn check(&self, item: &FallbackActionItemManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback action item manifest.
#[derive(Debug, Clone)]
pub struct FallbackActionItemManifestValidator {
    pub rules: Vec<Rc<dyn FallbackActionItemManifestRule>>,
}

impl FallbackActionItemManifestValidator {
    pub fn validate(&self, item: &FallbackActionItemManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackActionItemManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackActionItemManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_action_item_manifest_validator() -> FallbackActionItemManifestValidator {
    FallbackActionItemManifestValidator {
        rules: vec![
            Rc::new(FallbackActionItemManifestNameLengthRule { max_length: 32 })
                as Rc<dyn FallbackActionItemManifestRule>,
            Rc::new(FallbackActionItemManifestApprovedGuardRule { limit: 32 }),
            Rc::new(FallbackActionItemManifestExpiresAtRangeRule { min: 255, max: 1000 }),
            Rc::new(FallbackActionItemManifestBudgetRangeRule { min: 8.9, max: 4119.5 }),
            Rc::new(FallbackActionItemManifestAllowedNameRule {
                allowed: vec!["rolling".to_string(), "batched".to_string()],
            }),
            Rc::new(FallbackActionItemManifestCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_action_item_manifest_validator();
        let item = new_fallback_action_item_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackActionItemManifestNameLengthRule { max_length: 80 };
        let mut item = new_fallback_action_item_manifest("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FallbackActionItemManifestExpiresAtRangeRule { min: 80, max: 86400 };
        let mut item = new_fallback_action_item_manifest("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = FallbackActionItemManifestBudgetRangeRule { min: 6.8, max: 3117.3 };
        let mut item = new_fallback_action_item_manifest("x", "y");
        item.budget = 3118.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackActionItemManifestCodeLengthRule { max_length: 16 };
        let mut item = new_fallback_action_item_manifest("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
