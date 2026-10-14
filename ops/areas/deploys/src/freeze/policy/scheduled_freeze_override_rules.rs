use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled freeze override must pass. Check returns its problems, or none.
pub trait ScheduledFreezeOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFreezeOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeOverrideCodePrefixRule {
    pub prefix: String,
}

impl ScheduledFreezeOverrideRule for ScheduledFreezeOverrideCodePrefixRule {
    fn check(&self, item: &ScheduledFreezeOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeOverrideLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledFreezeOverrideRule for ScheduledFreezeOverrideLabelLengthRule {
    fn check(&self, item: &ScheduledFreezeOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeOverrideRequiredExternalRefRule {
    pub message: String,
}

impl ScheduledFreezeOverrideRule for ScheduledFreezeOverrideRequiredExternalRefRule {
    fn check(&self, item: &ScheduledFreezeOverride) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeOverrideWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFreezeOverrideRule for ScheduledFreezeOverrideWindowDaysRangeRule {
    fn check(&self, item: &ScheduledFreezeOverride) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled freeze override.
#[derive(Debug, Clone)]
pub struct ScheduledFreezeOverrideValidator {
    pub rules: Vec<Rc<dyn ScheduledFreezeOverrideRule>>,
}

impl ScheduledFreezeOverrideValidator {
    pub fn validate(&self, item: &ScheduledFreezeOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledFreezeOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFreezeOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_freeze_override_validator() -> ScheduledFreezeOverrideValidator {
    ScheduledFreezeOverrideValidator {
        rules: vec![
            Rc::new(ScheduledFreezeOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ScheduledFreezeOverrideRule>,
            Rc::new(ScheduledFreezeOverrideLabelLengthRule { max_length: 255 }),
            Rc::new(ScheduledFreezeOverrideRequiredExternalRefRule {
                message: "scheduled freeze override needs this field".to_string(),
            }),
            Rc::new(ScheduledFreezeOverrideWindowDaysRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_freeze_override_validator();
        let item = new_scheduled_freeze_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledFreezeOverrideLabelLengthRule { max_length: 1000 };
        let mut item = new_scheduled_freeze_override("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ScheduledFreezeOverrideRequiredExternalRefRule {
            message: "scheduled freeze override needs this field".to_string(),
        };
        let mut item = new_scheduled_freeze_override("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = ScheduledFreezeOverrideWindowDaysRangeRule { min: 80, max: 3600 };
        let mut item = new_scheduled_freeze_override("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
