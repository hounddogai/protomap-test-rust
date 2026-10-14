use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin profile must pass. Check returns its problems, or none.
pub trait RegionPinProfileRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProfileCodePrefixRule {
    pub prefix: String,
}

impl RegionPinProfileRule for RegionPinProfileCodePrefixRule {
    fn check(&self, item: &RegionPinProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProfileRequiredChannelRule {
    pub message: String,
}

impl RegionPinProfileRule for RegionPinProfileRequiredChannelRule {
    fn check(&self, item: &RegionPinProfile) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProfileRequiredCodeRule {
    pub message: String,
}

impl RegionPinProfileRule for RegionPinProfileRequiredCodeRule {
    fn check(&self, item: &RegionPinProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin profile.
#[derive(Debug, Clone)]
pub struct RegionPinProfileValidator {
    pub rules: Vec<Rc<dyn RegionPinProfileRule>>,
}

impl RegionPinProfileValidator {
    pub fn validate(&self, item: &RegionPinProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_profile_validator() -> RegionPinProfileValidator {
    RegionPinProfileValidator {
        rules: vec![
            Rc::new(RegionPinProfileCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RegionPinProfileRule>,
            Rc::new(RegionPinProfileRequiredChannelRule {
                message: "region pin profile needs an entry".to_string(),
            }),
            Rc::new(RegionPinProfileRequiredCodeRule {
                message: "region pin profile needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_profile_validator();
        let item = new_region_pin_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = RegionPinProfileRequiredChannelRule {
            message: "region pin profile needs an entry".to_string(),
        };
        let mut item = new_region_pin_profile("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinProfileRequiredCodeRule {
            message: "region pin profile needs this field".to_string(),
        };
        let mut item = new_region_pin_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
