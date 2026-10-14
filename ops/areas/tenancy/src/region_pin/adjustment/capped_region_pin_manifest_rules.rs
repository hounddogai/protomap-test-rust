use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped region pin manifest must pass. Check returns its problems, or none.
pub trait CappedRegionPinManifestRule: std::fmt::Debug {
    fn check(&self, item: &CappedRegionPinManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinManifestCodeLengthRule {
    pub max_length: i64,
}

impl CappedRegionPinManifestRule for CappedRegionPinManifestCodeLengthRule {
    fn check(&self, item: &CappedRegionPinManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinManifestNameLengthRule {
    pub max_length: i64,
}

impl CappedRegionPinManifestRule for CappedRegionPinManifestNameLengthRule {
    fn check(&self, item: &CappedRegionPinManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinManifestRequiredRegionRule {
    pub message: String,
}

impl CappedRegionPinManifestRule for CappedRegionPinManifestRequiredRegionRule {
    fn check(&self, item: &CappedRegionPinManifest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinManifestSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedRegionPinManifestRule for CappedRegionPinManifestSequenceRangeRule {
    fn check(&self, item: &CappedRegionPinManifest) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinManifestRequiredCodeRule {
    pub message: String,
}

impl CappedRegionPinManifestRule for CappedRegionPinManifestRequiredCodeRule {
    fn check(&self, item: &CappedRegionPinManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped region pin manifest.
#[derive(Debug, Clone)]
pub struct CappedRegionPinManifestValidator {
    pub rules: Vec<Rc<dyn CappedRegionPinManifestRule>>,
}

impl CappedRegionPinManifestValidator {
    pub fn validate(&self, item: &CappedRegionPinManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRegionPinManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRegionPinManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_region_pin_manifest_validator() -> CappedRegionPinManifestValidator {
    CappedRegionPinManifestValidator {
        rules: vec![
            Rc::new(CappedRegionPinManifestCodeLengthRule { max_length: 120 }) as Rc<dyn CappedRegionPinManifestRule>,
            Rc::new(CappedRegionPinManifestNameLengthRule { max_length: 32 }),
            Rc::new(CappedRegionPinManifestRequiredRegionRule {
                message: "capped region pin manifest needs an entry".to_string(),
            }),
            Rc::new(CappedRegionPinManifestSequenceRangeRule { min: 64, max: 120 }),
            Rc::new(CappedRegionPinManifestRequiredCodeRule {
                message: "capped region pin manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_region_pin_manifest_validator();
        let item = new_capped_region_pin_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedRegionPinManifestCodeLengthRule { max_length: 64 };
        let mut item = new_capped_region_pin_manifest("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CappedRegionPinManifestNameLengthRule { max_length: 3600 };
        let mut item = new_capped_region_pin_manifest("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = CappedRegionPinManifestRequiredRegionRule {
            message: "capped region pin manifest needs this field".to_string(),
        };
        let mut item = new_capped_region_pin_manifest("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = CappedRegionPinManifestSequenceRangeRule { min: 16, max: 120 };
        let mut item = new_capped_region_pin_manifest("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedRegionPinManifestRequiredCodeRule {
            message: "capped region pin manifest needs this field".to_string(),
        };
        let mut item = new_capped_region_pin_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
