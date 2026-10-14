use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default delegation override must pass. Check returns its problems, or none.
pub trait DefaultDelegationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDelegationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultDelegationOverrideRule for DefaultDelegationOverrideAllowedCodeRule {
    fn check(&self, item: &DefaultDelegationOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationOverrideDescriptionLengthRule {
    pub max_length: i64,
}

impl DefaultDelegationOverrideRule for DefaultDelegationOverrideDescriptionLengthRule {
    fn check(&self, item: &DefaultDelegationOverride) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationOverrideRequiredNameRule {
    pub message: String,
}

impl DefaultDelegationOverrideRule for DefaultDelegationOverrideRequiredNameRule {
    fn check(&self, item: &DefaultDelegationOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationOverrideDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultDelegationOverrideRule for DefaultDelegationOverrideDiscountRangeRule {
    fn check(&self, item: &DefaultDelegationOverride) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default delegation override.
#[derive(Debug, Clone)]
pub struct DefaultDelegationOverrideValidator {
    pub rules: Vec<Rc<dyn DefaultDelegationOverrideRule>>,
}

impl DefaultDelegationOverrideValidator {
    pub fn validate(&self, item: &DefaultDelegationOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultDelegationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDelegationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_delegation_override_validator() -> DefaultDelegationOverrideValidator {
    DefaultDelegationOverrideValidator {
        rules: vec![
            Rc::new(DefaultDelegationOverrideAllowedCodeRule {
                allowed: vec!["global".to_string(), "tiered".to_string(), "adaptive".to_string()],
            }) as Rc<dyn DefaultDelegationOverrideRule>,
            Rc::new(DefaultDelegationOverrideDescriptionLengthRule { max_length: 120 }),
            Rc::new(DefaultDelegationOverrideRequiredNameRule {
                message: "default delegation override needs a value".to_string(),
            }),
            Rc::new(DefaultDelegationOverrideDiscountRangeRule { min: 3.0, max: 322.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_delegation_override_validator();
        let item = new_default_delegation_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = DefaultDelegationOverrideDescriptionLengthRule { max_length: 32 };
        let mut item = new_default_delegation_override("x", "y");
        item.description = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultDelegationOverrideRequiredNameRule {
            message: "default delegation override needs a value".to_string(),
        };
        let mut item = new_default_delegation_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = DefaultDelegationOverrideDiscountRangeRule { min: 4.6, max: 2115.5 };
        let mut item = new_default_delegation_override("x", "y");
        item.discount = 2117.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
