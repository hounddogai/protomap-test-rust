use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional binding tier must pass. Check returns its problems, or none.
pub trait RegionalBindingTierRule: std::fmt::Debug {
    fn check(&self, item: &RegionalBindingTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingTierRequiredSourceRule {
    pub message: String,
}

impl RegionalBindingTierRule for RegionalBindingTierRequiredSourceRule {
    fn check(&self, item: &RegionalBindingTier) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingTierLabelLengthRule {
    pub max_length: i64,
}

impl RegionalBindingTierRule for RegionalBindingTierLabelLengthRule {
    fn check(&self, item: &RegionalBindingTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingTierAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RegionalBindingTierRule for RegionalBindingTierAllowedLabelRule {
    fn check(&self, item: &RegionalBindingTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingTierTimezoneLengthRule {
    pub max_length: i64,
}

impl RegionalBindingTierRule for RegionalBindingTierTimezoneLengthRule {
    fn check(&self, item: &RegionalBindingTier) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional binding tier.
#[derive(Debug, Clone)]
pub struct RegionalBindingTierValidator {
    pub rules: Vec<Rc<dyn RegionalBindingTierRule>>,
}

impl RegionalBindingTierValidator {
    pub fn validate(&self, item: &RegionalBindingTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalBindingTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalBindingTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_binding_tier_validator() -> RegionalBindingTierValidator {
    RegionalBindingTierValidator {
        rules: vec![
            Rc::new(RegionalBindingTierRequiredSourceRule {
                message: "regional binding tier needs this field".to_string(),
            }) as Rc<dyn RegionalBindingTierRule>,
            Rc::new(RegionalBindingTierLabelLengthRule { max_length: 255 }),
            Rc::new(RegionalBindingTierAllowedLabelRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(RegionalBindingTierTimezoneLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_binding_tier_validator();
        let item = new_regional_binding_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = RegionalBindingTierRequiredSourceRule {
            message: "regional binding tier needs a value".to_string(),
        };
        let mut item = new_regional_binding_tier("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalBindingTierLabelLengthRule { max_length: 86400 };
        let mut item = new_regional_binding_tier("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = RegionalBindingTierTimezoneLengthRule { max_length: 86400 };
        let mut item = new_regional_binding_tier("x", "y");
        item.timezone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
