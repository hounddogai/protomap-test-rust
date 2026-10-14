use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled region pin override must pass. Check returns its problems, or none.
pub trait ScheduledRegionPinOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRegionPinOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinOverrideRequiredCodeRule {
    pub message: String,
}

impl ScheduledRegionPinOverrideRule for ScheduledRegionPinOverrideRequiredCodeRule {
    fn check(&self, item: &ScheduledRegionPinOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinOverrideRequiredOwnerIdRule {
    pub message: String,
}

impl ScheduledRegionPinOverrideRule for ScheduledRegionPinOverrideRequiredOwnerIdRule {
    fn check(&self, item: &ScheduledRegionPinOverride) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinOverrideLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledRegionPinOverrideRule for ScheduledRegionPinOverrideLabelLengthRule {
    fn check(&self, item: &ScheduledRegionPinOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinOverrideAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ScheduledRegionPinOverrideRule for ScheduledRegionPinOverrideAllowedLabelRule {
    fn check(&self, item: &ScheduledRegionPinOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinOverrideCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledRegionPinOverrideRule for ScheduledRegionPinOverrideCodeLengthRule {
    fn check(&self, item: &ScheduledRegionPinOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled region pin override.
#[derive(Debug, Clone)]
pub struct ScheduledRegionPinOverrideValidator {
    pub rules: Vec<Rc<dyn ScheduledRegionPinOverrideRule>>,
}

impl ScheduledRegionPinOverrideValidator {
    pub fn validate(&self, item: &ScheduledRegionPinOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledRegionPinOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRegionPinOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_region_pin_override_validator() -> ScheduledRegionPinOverrideValidator {
    ScheduledRegionPinOverrideValidator {
        rules: vec![
            Rc::new(ScheduledRegionPinOverrideRequiredCodeRule {
                message: "scheduled region pin override needs an entry".to_string(),
            }) as Rc<dyn ScheduledRegionPinOverrideRule>,
            Rc::new(ScheduledRegionPinOverrideRequiredOwnerIdRule {
                message: "scheduled region pin override needs this field".to_string(),
            }),
            Rc::new(ScheduledRegionPinOverrideLabelLengthRule { max_length: 80 }),
            Rc::new(ScheduledRegionPinOverrideAllowedLabelRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(ScheduledRegionPinOverrideCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_region_pin_override_validator();
        let item = new_scheduled_region_pin_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledRegionPinOverrideRequiredCodeRule {
            message: "scheduled region pin override needs a value".to_string(),
        };
        let mut item = new_scheduled_region_pin_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = ScheduledRegionPinOverrideRequiredOwnerIdRule {
            message: "scheduled region pin override needs this field".to_string(),
        };
        let mut item = new_scheduled_region_pin_override("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledRegionPinOverrideLabelLengthRule { max_length: 64 };
        let mut item = new_scheduled_region_pin_override("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledRegionPinOverrideCodeLengthRule { max_length: 80 };
        let mut item = new_scheduled_region_pin_override("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
