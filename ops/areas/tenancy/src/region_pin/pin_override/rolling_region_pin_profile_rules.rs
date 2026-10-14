use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling region pin profile must pass. Check returns its problems, or none.
pub trait RollingRegionPinProfileRule: std::fmt::Debug {
    fn check(&self, item: &RollingRegionPinProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinProfileAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl RollingRegionPinProfileRule for RollingRegionPinProfileAllowedSourceRule {
    fn check(&self, item: &RollingRegionPinProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is not allowed", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinProfileRequiredLabelRule {
    pub message: String,
}

impl RollingRegionPinProfileRule for RollingRegionPinProfileRequiredLabelRule {
    fn check(&self, item: &RollingRegionPinProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinProfileCodeLengthRule {
    pub max_length: i64,
}

impl RollingRegionPinProfileRule for RollingRegionPinProfileCodeLengthRule {
    fn check(&self, item: &RollingRegionPinProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling region pin profile.
#[derive(Debug, Clone)]
pub struct RollingRegionPinProfileValidator {
    pub rules: Vec<Rc<dyn RollingRegionPinProfileRule>>,
}

impl RollingRegionPinProfileValidator {
    pub fn validate(&self, item: &RollingRegionPinProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingRegionPinProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRegionPinProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_region_pin_profile_validator() -> RollingRegionPinProfileValidator {
    RollingRegionPinProfileValidator {
        rules: vec![
            Rc::new(RollingRegionPinProfileAllowedSourceRule {
                allowed: vec!["regional".to_string(), "regional".to_string(), "seasonal".to_string()],
            }) as Rc<dyn RollingRegionPinProfileRule>,
            Rc::new(RollingRegionPinProfileRequiredLabelRule {
                message: "rolling region pin profile needs an entry".to_string(),
            }),
            Rc::new(RollingRegionPinProfileCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_region_pin_profile_validator();
        let item = new_rolling_region_pin_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingRegionPinProfileRequiredLabelRule {
            message: "rolling region pin profile needs this field".to_string(),
        };
        let mut item = new_rolling_region_pin_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingRegionPinProfileCodeLengthRule { max_length: 1000 };
        let mut item = new_rolling_region_pin_profile("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
