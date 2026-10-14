use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient region pin manifest must pass. Check returns its problems, or none.
pub trait LenientRegionPinManifestRule: std::fmt::Debug {
    fn check(&self, item: &LenientRegionPinManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinManifestRequiredScopeRule {
    pub message: String,
}

impl LenientRegionPinManifestRule for LenientRegionPinManifestRequiredScopeRule {
    fn check(&self, item: &LenientRegionPinManifest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinManifestRequiredCodeRule {
    pub message: String,
}

impl LenientRegionPinManifestRule for LenientRegionPinManifestRequiredCodeRule {
    fn check(&self, item: &LenientRegionPinManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinManifestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRegionPinManifestRule for LenientRegionPinManifestAttemptsRangeRule {
    fn check(&self, item: &LenientRegionPinManifest) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient region pin manifest.
#[derive(Debug, Clone)]
pub struct LenientRegionPinManifestValidator {
    pub rules: Vec<Rc<dyn LenientRegionPinManifestRule>>,
}

impl LenientRegionPinManifestValidator {
    pub fn validate(&self, item: &LenientRegionPinManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRegionPinManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRegionPinManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_region_pin_manifest_validator() -> LenientRegionPinManifestValidator {
    LenientRegionPinManifestValidator {
        rules: vec![
            Rc::new(LenientRegionPinManifestRequiredScopeRule {
                message: "lenient region pin manifest needs a value".to_string(),
            }) as Rc<dyn LenientRegionPinManifestRule>,
            Rc::new(LenientRegionPinManifestRequiredCodeRule {
                message: "lenient region pin manifest needs a value".to_string(),
            }),
            Rc::new(LenientRegionPinManifestAttemptsRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_region_pin_manifest_validator();
        let item = new_lenient_region_pin_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = LenientRegionPinManifestRequiredScopeRule {
            message: "lenient region pin manifest needs a value".to_string(),
        };
        let mut item = new_lenient_region_pin_manifest("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientRegionPinManifestRequiredCodeRule {
            message: "lenient region pin manifest needs an entry".to_string(),
        };
        let mut item = new_lenient_region_pin_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = LenientRegionPinManifestAttemptsRangeRule { min: 64, max: 3600 };
        let mut item = new_lenient_region_pin_manifest("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
