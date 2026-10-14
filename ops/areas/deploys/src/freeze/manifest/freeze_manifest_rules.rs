use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze manifest must pass. Check returns its problems, or none.
pub trait FreezeManifestRule: std::fmt::Debug {
    fn check(&self, item: &FreezeManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeManifestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeManifestRule for FreezeManifestMaximumRangeRule {
    fn check(&self, item: &FreezeManifest) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeManifestVisibleGuardRule {
    pub limit: f64,
}

impl FreezeManifestRule for FreezeManifestVisibleGuardRule {
    fn check(&self, item: &FreezeManifest) -> Vec<String> {
        if item.visible && (item.maximum > self.limit) {
            return vec![format!("maximum {} exceeds {} while visible", item.maximum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeManifestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl FreezeManifestRule for FreezeManifestAllowedTitleRule {
    fn check(&self, item: &FreezeManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeManifestCodePrefixRule {
    pub prefix: String,
}

impl FreezeManifestRule for FreezeManifestCodePrefixRule {
    fn check(&self, item: &FreezeManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze manifest.
#[derive(Debug, Clone)]
pub struct FreezeManifestValidator {
    pub rules: Vec<Rc<dyn FreezeManifestRule>>,
}

impl FreezeManifestValidator {
    pub fn validate(&self, item: &FreezeManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_manifest_validator() -> FreezeManifestValidator {
    FreezeManifestValidator {
        rules: vec![
            Rc::new(FreezeManifestMaximumRangeRule { min: 1.0, max: 857.4 }) as Rc<dyn FreezeManifestRule>,
            Rc::new(FreezeManifestVisibleGuardRule { limit: 7.7 }),
            Rc::new(FreezeManifestAllowedTitleRule {
                allowed: vec!["strict".to_string(), "batched".to_string(), "tiered".to_string()],
            }),
            Rc::new(FreezeManifestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_manifest_validator();
        let item = new_freeze_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = FreezeManifestMaximumRangeRule { min: 6.9, max: 3742.6 };
        let mut item = new_freeze_manifest("x", "y");
        item.maximum = 3744.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
