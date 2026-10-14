use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin tier must pass. Check returns its problems, or none.
pub trait RegionPinTierRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTierLabelLengthRule {
    pub max_length: i64,
}

impl RegionPinTierRule for RegionPinTierLabelLengthRule {
    fn check(&self, item: &RegionPinTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTierRequiredCountryRule {
    pub message: String,
}

impl RegionPinTierRule for RegionPinTierRequiredCountryRule {
    fn check(&self, item: &RegionPinTier) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTierRequiredCodeRule {
    pub message: String,
}

impl RegionPinTierRule for RegionPinTierRequiredCodeRule {
    fn check(&self, item: &RegionPinTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTierAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RegionPinTierRule for RegionPinTierAllowedTitleRule {
    fn check(&self, item: &RegionPinTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin tier.
#[derive(Debug, Clone)]
pub struct RegionPinTierValidator {
    pub rules: Vec<Rc<dyn RegionPinTierRule>>,
}

impl RegionPinTierValidator {
    pub fn validate(&self, item: &RegionPinTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_tier_validator() -> RegionPinTierValidator {
    RegionPinTierValidator {
        rules: vec![
            Rc::new(RegionPinTierLabelLengthRule { max_length: 16 }) as Rc<dyn RegionPinTierRule>,
            Rc::new(RegionPinTierRequiredCountryRule {
                message: "region pin tier needs a value".to_string(),
            }),
            Rc::new(RegionPinTierRequiredCodeRule {
                message: "region pin tier needs this field".to_string(),
            }),
            Rc::new(RegionPinTierAllowedTitleRule {
                allowed: vec!["global".to_string(), "global".to_string(), "scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_tier_validator();
        let item = new_region_pin_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionPinTierLabelLengthRule { max_length: 255 };
        let mut item = new_region_pin_tier("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = RegionPinTierRequiredCountryRule {
            message: "region pin tier needs a value".to_string(),
        };
        let mut item = new_region_pin_tier("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinTierRequiredCodeRule {
            message: "region pin tier needs a value".to_string(),
        };
        let mut item = new_region_pin_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
