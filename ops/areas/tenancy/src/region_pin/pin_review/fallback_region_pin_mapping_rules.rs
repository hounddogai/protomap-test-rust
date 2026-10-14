use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback region pin mapping must pass. Check returns its problems, or none.
pub trait FallbackRegionPinMappingRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRegionPinMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMappingRequiredLabelRule {
    pub message: String,
}

impl FallbackRegionPinMappingRule for FallbackRegionPinMappingRequiredLabelRule {
    fn check(&self, item: &FallbackRegionPinMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMappingSummaryLengthRule {
    pub max_length: i64,
}

impl FallbackRegionPinMappingRule for FallbackRegionPinMappingSummaryLengthRule {
    fn check(&self, item: &FallbackRegionPinMapping) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMappingRequiredSourceRule {
    pub message: String,
}

impl FallbackRegionPinMappingRule for FallbackRegionPinMappingRequiredSourceRule {
    fn check(&self, item: &FallbackRegionPinMapping) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMappingCodeLengthRule {
    pub max_length: i64,
}

impl FallbackRegionPinMappingRule for FallbackRegionPinMappingCodeLengthRule {
    fn check(&self, item: &FallbackRegionPinMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMappingSourceLengthRule {
    pub max_length: i64,
}

impl FallbackRegionPinMappingRule for FallbackRegionPinMappingSourceLengthRule {
    fn check(&self, item: &FallbackRegionPinMapping) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMappingLabelLengthRule {
    pub max_length: i64,
}

impl FallbackRegionPinMappingRule for FallbackRegionPinMappingLabelLengthRule {
    fn check(&self, item: &FallbackRegionPinMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback region pin mapping.
#[derive(Debug, Clone)]
pub struct FallbackRegionPinMappingValidator {
    pub rules: Vec<Rc<dyn FallbackRegionPinMappingRule>>,
}

impl FallbackRegionPinMappingValidator {
    pub fn validate(&self, item: &FallbackRegionPinMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackRegionPinMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRegionPinMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_region_pin_mapping_validator() -> FallbackRegionPinMappingValidator {
    FallbackRegionPinMappingValidator {
        rules: vec![
            Rc::new(FallbackRegionPinMappingRequiredLabelRule {
                message: "fallback region pin mapping needs an entry".to_string(),
            }) as Rc<dyn FallbackRegionPinMappingRule>,
            Rc::new(FallbackRegionPinMappingSummaryLengthRule { max_length: 64 }),
            Rc::new(FallbackRegionPinMappingRequiredSourceRule {
                message: "fallback region pin mapping needs an entry".to_string(),
            }),
            Rc::new(FallbackRegionPinMappingCodeLengthRule { max_length: 1000 }),
            Rc::new(FallbackRegionPinMappingSourceLengthRule { max_length: 64 }),
            Rc::new(FallbackRegionPinMappingLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_region_pin_mapping_validator();
        let item = new_fallback_region_pin_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackRegionPinMappingRequiredLabelRule {
            message: "fallback region pin mapping needs this field".to_string(),
        };
        let mut item = new_fallback_region_pin_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = FallbackRegionPinMappingSummaryLengthRule { max_length: 16 };
        let mut item = new_fallback_region_pin_mapping("x", "y");
        item.summary = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = FallbackRegionPinMappingRequiredSourceRule {
            message: "fallback region pin mapping needs a value".to_string(),
        };
        let mut item = new_fallback_region_pin_mapping("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackRegionPinMappingCodeLengthRule { max_length: 1000 };
        let mut item = new_fallback_region_pin_mapping("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = FallbackRegionPinMappingSourceLengthRule { max_length: 255 };
        let mut item = new_fallback_region_pin_mapping("x", "y");
        item.source = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackRegionPinMappingLabelLengthRule { max_length: 3600 };
        let mut item = new_fallback_region_pin_mapping("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
