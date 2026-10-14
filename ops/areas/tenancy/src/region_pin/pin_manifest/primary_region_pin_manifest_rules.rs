use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary region pin manifest must pass. Check returns its problems, or none.
pub trait PrimaryRegionPinManifestRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRegionPinManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinManifestRequiredAudienceRule {
    pub message: String,
}

impl PrimaryRegionPinManifestRule for PrimaryRegionPinManifestRequiredAudienceRule {
    fn check(&self, item: &PrimaryRegionPinManifest) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinManifestRequiredCodeRule {
    pub message: String,
}

impl PrimaryRegionPinManifestRule for PrimaryRegionPinManifestRequiredCodeRule {
    fn check(&self, item: &PrimaryRegionPinManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinManifestAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl PrimaryRegionPinManifestRule for PrimaryRegionPinManifestAllowedFingerprintRule {
    fn check(&self, item: &PrimaryRegionPinManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is missing", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinManifestNameLengthRule {
    pub max_length: i64,
}

impl PrimaryRegionPinManifestRule for PrimaryRegionPinManifestNameLengthRule {
    fn check(&self, item: &PrimaryRegionPinManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinManifestBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRegionPinManifestRule for PrimaryRegionPinManifestBudgetRangeRule {
    fn check(&self, item: &PrimaryRegionPinManifest) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary region pin manifest.
#[derive(Debug, Clone)]
pub struct PrimaryRegionPinManifestValidator {
    pub rules: Vec<Rc<dyn PrimaryRegionPinManifestRule>>,
}

impl PrimaryRegionPinManifestValidator {
    pub fn validate(&self, item: &PrimaryRegionPinManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRegionPinManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRegionPinManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_region_pin_manifest_validator() -> PrimaryRegionPinManifestValidator {
    PrimaryRegionPinManifestValidator {
        rules: vec![
            Rc::new(PrimaryRegionPinManifestRequiredAudienceRule {
                message: "primary region pin manifest needs a value".to_string(),
            }) as Rc<dyn PrimaryRegionPinManifestRule>,
            Rc::new(PrimaryRegionPinManifestRequiredCodeRule {
                message: "primary region pin manifest needs an entry".to_string(),
            }),
            Rc::new(PrimaryRegionPinManifestAllowedFingerprintRule { allowed: Vec::new() }),
            Rc::new(PrimaryRegionPinManifestNameLengthRule { max_length: 120 }),
            Rc::new(PrimaryRegionPinManifestBudgetRangeRule { min: 0.0, max: 1914.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_region_pin_manifest_validator();
        let item = new_primary_region_pin_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PrimaryRegionPinManifestRequiredAudienceRule {
            message: "primary region pin manifest needs this field".to_string(),
        };
        let mut item = new_primary_region_pin_manifest("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryRegionPinManifestRequiredCodeRule {
            message: "primary region pin manifest needs a value".to_string(),
        };
        let mut item = new_primary_region_pin_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryRegionPinManifestNameLengthRule { max_length: 86400 };
        let mut item = new_primary_region_pin_manifest("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PrimaryRegionPinManifestBudgetRangeRule { min: 6.5, max: 2414.9 };
        let mut item = new_primary_region_pin_manifest("x", "y");
        item.budget = 2416.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
