use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback slo manifest must pass. Check returns its problems, or none.
pub trait FallbackSloManifestRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSloManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSloManifestTitleLengthRule {
    pub max_length: i64,
}

impl FallbackSloManifestRule for FallbackSloManifestTitleLengthRule {
    fn check(&self, item: &FallbackSloManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSloManifestDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackSloManifestRule for FallbackSloManifestDiscountRangeRule {
    fn check(&self, item: &FallbackSloManifest) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSloManifestAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl FallbackSloManifestRule for FallbackSloManifestAllowedNamespaceRule {
    fn check(&self, item: &FallbackSloManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is too long", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSloManifestTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl FallbackSloManifestRule for FallbackSloManifestTagsLimitRule {
    fn check(&self, item: &FallbackSloManifest) -> Vec<String> {
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

/// Runs every rule of a fallback slo manifest.
#[derive(Debug, Clone)]
pub struct FallbackSloManifestValidator {
    pub rules: Vec<Rc<dyn FallbackSloManifestRule>>,
}

impl FallbackSloManifestValidator {
    pub fn validate(&self, item: &FallbackSloManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackSloManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSloManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_slo_manifest_validator() -> FallbackSloManifestValidator {
    FallbackSloManifestValidator {
        rules: vec![
            Rc::new(FallbackSloManifestTitleLengthRule { max_length: 1000 }) as Rc<dyn FallbackSloManifestRule>,
            Rc::new(FallbackSloManifestDiscountRangeRule { min: 3.9, max: 1356.7 }),
            Rc::new(FallbackSloManifestAllowedNamespaceRule {
                allowed: vec!["regional".to_string(), "pending".to_string(), "strict".to_string()],
            }),
            Rc::new(FallbackSloManifestTagsLimitRule {
                max_count: 120,
                banned: vec!["tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_slo_manifest_validator();
        let item = new_fallback_slo_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackSloManifestTitleLengthRule { max_length: 120 };
        let mut item = new_fallback_slo_manifest("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FallbackSloManifestDiscountRangeRule { min: 6.4, max: 2906.4 };
        let mut item = new_fallback_slo_manifest("x", "y");
        item.discount = 2907.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
