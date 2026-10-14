use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin baseline must pass. Check returns its problems, or none.
pub trait RegionPinBaselineRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaselineRequiredExternalRefRule {
    pub message: String,
}

impl RegionPinBaselineRule for RegionPinBaselineRequiredExternalRefRule {
    fn check(&self, item: &RegionPinBaseline) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaselineCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinBaselineRule for RegionPinBaselineCodeLengthRule {
    fn check(&self, item: &RegionPinBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaselineAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RegionPinBaselineRule for RegionPinBaselineAllowedLabelRule {
    fn check(&self, item: &RegionPinBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaselineLabelLengthRule {
    pub max_length: i64,
}

impl RegionPinBaselineRule for RegionPinBaselineLabelLengthRule {
    fn check(&self, item: &RegionPinBaseline) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaselineRequiredLabelRule {
    pub message: String,
}

impl RegionPinBaselineRule for RegionPinBaselineRequiredLabelRule {
    fn check(&self, item: &RegionPinBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin baseline.
#[derive(Debug, Clone)]
pub struct RegionPinBaselineValidator {
    pub rules: Vec<Rc<dyn RegionPinBaselineRule>>,
}

impl RegionPinBaselineValidator {
    pub fn validate(&self, item: &RegionPinBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_baseline_validator() -> RegionPinBaselineValidator {
    RegionPinBaselineValidator {
        rules: vec![
            Rc::new(RegionPinBaselineRequiredExternalRefRule {
                message: "region pin baseline needs a value".to_string(),
            }) as Rc<dyn RegionPinBaselineRule>,
            Rc::new(RegionPinBaselineCodeLengthRule { max_length: 32 }),
            Rc::new(RegionPinBaselineAllowedLabelRule {
                allowed: vec!["legacy".to_string(), "seasonal".to_string()],
            }),
            Rc::new(RegionPinBaselineLabelLengthRule { max_length: 32 }),
            Rc::new(RegionPinBaselineRequiredLabelRule {
                message: "region pin baseline needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_baseline_validator();
        let item = new_region_pin_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = RegionPinBaselineRequiredExternalRefRule {
            message: "region pin baseline needs this field".to_string(),
        };
        let mut item = new_region_pin_baseline("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinBaselineCodeLengthRule { max_length: 1000 };
        let mut item = new_region_pin_baseline("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionPinBaselineLabelLengthRule { max_length: 3600 };
        let mut item = new_region_pin_baseline("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionPinBaselineRequiredLabelRule {
            message: "region pin baseline needs this field".to_string(),
        };
        let mut item = new_region_pin_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
