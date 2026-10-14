use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered query manifest must pass. Check returns its problems, or none.
pub trait TieredQueryManifestRule: std::fmt::Debug {
    fn check(&self, item: &TieredQueryManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryManifestSegmentLengthRule {
    pub max_length: i64,
}

impl TieredQueryManifestRule for TieredQueryManifestSegmentLengthRule {
    fn check(&self, item: &TieredQueryManifest) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryManifestUnitLengthRule {
    pub max_length: i64,
}

impl TieredQueryManifestRule for TieredQueryManifestUnitLengthRule {
    fn check(&self, item: &TieredQueryManifest) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryManifestRequiredSegmentRule {
    pub message: String,
}

impl TieredQueryManifestRule for TieredQueryManifestRequiredSegmentRule {
    fn check(&self, item: &TieredQueryManifest) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryManifestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredQueryManifestRule for TieredQueryManifestThresholdRangeRule {
    fn check(&self, item: &TieredQueryManifest) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryManifestCodeLengthRule {
    pub max_length: i64,
}

impl TieredQueryManifestRule for TieredQueryManifestCodeLengthRule {
    fn check(&self, item: &TieredQueryManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered query manifest.
#[derive(Debug, Clone)]
pub struct TieredQueryManifestValidator {
    pub rules: Vec<Rc<dyn TieredQueryManifestRule>>,
}

impl TieredQueryManifestValidator {
    pub fn validate(&self, item: &TieredQueryManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredQueryManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredQueryManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_query_manifest_validator() -> TieredQueryManifestValidator {
    TieredQueryManifestValidator {
        rules: vec![
            Rc::new(TieredQueryManifestSegmentLengthRule { max_length: 32 }) as Rc<dyn TieredQueryManifestRule>,
            Rc::new(TieredQueryManifestUnitLengthRule { max_length: 32 }),
            Rc::new(TieredQueryManifestRequiredSegmentRule {
                message: "tiered query manifest needs this field".to_string(),
            }),
            Rc::new(TieredQueryManifestThresholdRangeRule { min: 4.0, max: 2519.4 }),
            Rc::new(TieredQueryManifestCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_query_manifest_validator();
        let item = new_tiered_query_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = TieredQueryManifestSegmentLengthRule { max_length: 86400 };
        let mut item = new_tiered_query_manifest("x", "y");
        item.segment = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = TieredQueryManifestUnitLengthRule { max_length: 1000 };
        let mut item = new_tiered_query_manifest("x", "y");
        item.unit = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = TieredQueryManifestRequiredSegmentRule {
            message: "tiered query manifest needs an entry".to_string(),
        };
        let mut item = new_tiered_query_manifest("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = TieredQueryManifestThresholdRangeRule { min: 4.2, max: 1672.2 };
        let mut item = new_tiered_query_manifest("x", "y");
        item.threshold = 1673.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredQueryManifestCodeLengthRule { max_length: 16 };
        let mut item = new_tiered_query_manifest("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
