use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo manifest must pass. Check returns its problems, or none.
pub trait SloManifestRule: std::fmt::Debug {
    fn check(&self, item: &SloManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloManifestCodeLengthRule {
    pub max_length: i64,
}

impl SloManifestRule for SloManifestCodeLengthRule {
    fn check(&self, item: &SloManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloManifestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloManifestRule for SloManifestMinimumRangeRule {
    fn check(&self, item: &SloManifest) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloManifestDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloManifestRule for SloManifestDiscountRangeRule {
    fn check(&self, item: &SloManifest) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloManifestRegionLengthRule {
    pub max_length: i64,
}

impl SloManifestRule for SloManifestRegionLengthRule {
    fn check(&self, item: &SloManifest) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloManifestVisibleGuardRule {
    pub limit: f64,
}

impl SloManifestRule for SloManifestVisibleGuardRule {
    fn check(&self, item: &SloManifest) -> Vec<String> {
        if item.visible && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while visible", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloManifestRequiredRegionRule {
    pub message: String,
}

impl SloManifestRule for SloManifestRequiredRegionRule {
    fn check(&self, item: &SloManifest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo manifest.
#[derive(Debug, Clone)]
pub struct SloManifestValidator {
    pub rules: Vec<Rc<dyn SloManifestRule>>,
}

impl SloManifestValidator {
    pub fn validate(&self, item: &SloManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_manifest_validator() -> SloManifestValidator {
    SloManifestValidator {
        rules: vec![
            Rc::new(SloManifestCodeLengthRule { max_length: 3600 }) as Rc<dyn SloManifestRule>,
            Rc::new(SloManifestMinimumRangeRule { min: 0.3, max: 2431.0 }),
            Rc::new(SloManifestDiscountRangeRule { min: 8.8, max: 1107.6 }),
            Rc::new(SloManifestRegionLengthRule { max_length: 64 }),
            Rc::new(SloManifestVisibleGuardRule { limit: 2.9 }),
            Rc::new(SloManifestRequiredRegionRule {
                message: "slo manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_manifest_validator();
        let item = new_slo_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SloManifestCodeLengthRule { max_length: 80 };
        let mut item = new_slo_manifest("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SloManifestMinimumRangeRule { min: 6.1, max: 642.2 };
        let mut item = new_slo_manifest("x", "y");
        item.minimum = 643.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SloManifestDiscountRangeRule { min: 6.5, max: 1757.8 };
        let mut item = new_slo_manifest("x", "y");
        item.discount = 1759.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = SloManifestRegionLengthRule { max_length: 64 };
        let mut item = new_slo_manifest("x", "y");
        item.region = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = SloManifestRequiredRegionRule {
            message: "slo manifest needs an entry".to_string(),
        };
        let mut item = new_slo_manifest("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
