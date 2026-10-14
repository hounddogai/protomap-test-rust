use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback overage manifest must pass. Check returns its problems, or none.
pub trait FallbackOverageManifestRule: std::fmt::Debug {
    fn check(&self, item: &FallbackOverageManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageManifestTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl FallbackOverageManifestRule for FallbackOverageManifestTagsLimitRule {
    fn check(&self, item: &FallbackOverageManifest) -> Vec<String> {
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
pub struct FallbackOverageManifestRequiredCodeRule {
    pub message: String,
}

impl FallbackOverageManifestRule for FallbackOverageManifestRequiredCodeRule {
    fn check(&self, item: &FallbackOverageManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageManifestLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackOverageManifestRule for FallbackOverageManifestLimitRangeRule {
    fn check(&self, item: &FallbackOverageManifest) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageManifestSummaryLengthRule {
    pub max_length: i64,
}

impl FallbackOverageManifestRule for FallbackOverageManifestSummaryLengthRule {
    fn check(&self, item: &FallbackOverageManifest) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback overage manifest.
#[derive(Debug, Clone)]
pub struct FallbackOverageManifestValidator {
    pub rules: Vec<Rc<dyn FallbackOverageManifestRule>>,
}

impl FallbackOverageManifestValidator {
    pub fn validate(&self, item: &FallbackOverageManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackOverageManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackOverageManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_overage_manifest_validator() -> FallbackOverageManifestValidator {
    FallbackOverageManifestValidator {
        rules: vec![
            Rc::new(FallbackOverageManifestTagsLimitRule {
                max_count: 64,
                banned: vec!["manual".to_string(), "rolling".to_string()],
            }) as Rc<dyn FallbackOverageManifestRule>,
            Rc::new(FallbackOverageManifestRequiredCodeRule {
                message: "fallback overage manifest needs an entry".to_string(),
            }),
            Rc::new(FallbackOverageManifestLimitRangeRule { min: 120, max: 1000 }),
            Rc::new(FallbackOverageManifestSummaryLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_overage_manifest_validator();
        let item = new_fallback_overage_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackOverageManifestRequiredCodeRule {
            message: "fallback overage manifest needs this field".to_string(),
        };
        let mut item = new_fallback_overage_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = FallbackOverageManifestLimitRangeRule { min: 32, max: 255 };
        let mut item = new_fallback_overage_manifest("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = FallbackOverageManifestSummaryLengthRule { max_length: 120 };
        let mut item = new_fallback_overage_manifest("x", "y");
        item.summary = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
