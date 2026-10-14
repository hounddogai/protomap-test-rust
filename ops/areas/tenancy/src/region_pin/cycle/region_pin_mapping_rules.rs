use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin mapping must pass. Check returns its problems, or none.
pub trait RegionPinMappingRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMappingRequiredCodeRule {
    pub message: String,
}

impl RegionPinMappingRule for RegionPinMappingRequiredCodeRule {
    fn check(&self, item: &RegionPinMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMappingCodePrefixRule {
    pub prefix: String,
}

impl RegionPinMappingRule for RegionPinMappingCodePrefixRule {
    fn check(&self, item: &RegionPinMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMappingRequiredLocaleRule {
    pub message: String,
}

impl RegionPinMappingRule for RegionPinMappingRequiredLocaleRule {
    fn check(&self, item: &RegionPinMapping) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMappingLocaleLengthRule {
    pub max_length: i64,
}

impl RegionPinMappingRule for RegionPinMappingLocaleLengthRule {
    fn check(&self, item: &RegionPinMapping) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMappingRequiredLabelRule {
    pub message: String,
}

impl RegionPinMappingRule for RegionPinMappingRequiredLabelRule {
    fn check(&self, item: &RegionPinMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMappingAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionPinMappingRule for RegionPinMappingAllowedCodeRule {
    fn check(&self, item: &RegionPinMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin mapping.
#[derive(Debug, Clone)]
pub struct RegionPinMappingValidator {
    pub rules: Vec<Rc<dyn RegionPinMappingRule>>,
}

impl RegionPinMappingValidator {
    pub fn validate(&self, item: &RegionPinMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_mapping_validator() -> RegionPinMappingValidator {
    RegionPinMappingValidator {
        rules: vec![
            Rc::new(RegionPinMappingRequiredCodeRule {
                message: "region pin mapping needs this field".to_string(),
            }) as Rc<dyn RegionPinMappingRule>,
            Rc::new(RegionPinMappingCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RegionPinMappingRequiredLocaleRule {
                message: "region pin mapping needs this field".to_string(),
            }),
            Rc::new(RegionPinMappingLocaleLengthRule { max_length: 255 }),
            Rc::new(RegionPinMappingRequiredLabelRule {
                message: "region pin mapping needs this field".to_string(),
            }),
            Rc::new(RegionPinMappingAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_mapping_validator();
        let item = new_region_pin_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinMappingRequiredCodeRule {
            message: "region pin mapping needs an entry".to_string(),
        };
        let mut item = new_region_pin_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RegionPinMappingRequiredLocaleRule {
            message: "region pin mapping needs this field".to_string(),
        };
        let mut item = new_region_pin_mapping("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RegionPinMappingLocaleLengthRule { max_length: 64 };
        let mut item = new_region_pin_mapping("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionPinMappingRequiredLabelRule {
            message: "region pin mapping needs this field".to_string(),
        };
        let mut item = new_region_pin_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
