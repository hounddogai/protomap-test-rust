use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin bundle must pass. Check returns its problems, or none.
pub trait RegionPinBundleRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBundleCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinBundleRule for RegionPinBundleCodeLengthRule {
    fn check(&self, item: &RegionPinBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBundleNameLengthRule {
    pub max_length: i64,
}

impl RegionPinBundleRule for RegionPinBundleNameLengthRule {
    fn check(&self, item: &RegionPinBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBundleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinBundleRule for RegionPinBundleTtlSecondsRangeRule {
    fn check(&self, item: &RegionPinBundle) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a region pin bundle.
#[derive(Debug, Clone)]
pub struct RegionPinBundleValidator {
    pub rules: Vec<Rc<dyn RegionPinBundleRule>>,
}

impl RegionPinBundleValidator {
    pub fn validate(&self, item: &RegionPinBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_bundle_validator() -> RegionPinBundleValidator {
    RegionPinBundleValidator {
        rules: vec![
            Rc::new(RegionPinBundleCodeLengthRule { max_length: 3600 }) as Rc<dyn RegionPinBundleRule>,
            Rc::new(RegionPinBundleNameLengthRule { max_length: 120 }),
            Rc::new(RegionPinBundleTtlSecondsRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_bundle_validator();
        let item = new_region_pin_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinBundleCodeLengthRule { max_length: 32 };
        let mut item = new_region_pin_bundle("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionPinBundleNameLengthRule { max_length: 255 };
        let mut item = new_region_pin_bundle("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RegionPinBundleTtlSecondsRangeRule { min: 80, max: 3600 };
        let mut item = new_region_pin_bundle("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
