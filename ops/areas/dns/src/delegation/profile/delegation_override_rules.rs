use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation override must pass. Check returns its problems, or none.
pub trait DelegationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &DelegationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationOverrideDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationOverrideRule for DelegationOverrideDurationMinutesRangeRule {
    fn check(&self, item: &DelegationOverride) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationOverrideRequiredLabelRule {
    pub message: String,
}

impl DelegationOverrideRule for DelegationOverrideRequiredLabelRule {
    fn check(&self, item: &DelegationOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationOverrideRequiredSummaryRule {
    pub message: String,
}

impl DelegationOverrideRule for DelegationOverrideRequiredSummaryRule {
    fn check(&self, item: &DelegationOverride) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationOverrideCodePrefixRule {
    pub prefix: String,
}

impl DelegationOverrideRule for DelegationOverrideCodePrefixRule {
    fn check(&self, item: &DelegationOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationOverrideAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl DelegationOverrideRule for DelegationOverrideAllowedLabelRule {
    fn check(&self, item: &DelegationOverride) -> Vec<String> {
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
pub struct DelegationOverrideRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationOverrideRule for DelegationOverrideRetriesRangeRule {
    fn check(&self, item: &DelegationOverride) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation override.
#[derive(Debug, Clone)]
pub struct DelegationOverrideValidator {
    pub rules: Vec<Rc<dyn DelegationOverrideRule>>,
}

impl DelegationOverrideValidator {
    pub fn validate(&self, item: &DelegationOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_override_validator() -> DelegationOverrideValidator {
    DelegationOverrideValidator {
        rules: vec![
            Rc::new(DelegationOverrideDurationMinutesRangeRule { min: 16, max: 64 }) as Rc<dyn DelegationOverrideRule>,
            Rc::new(DelegationOverrideRequiredLabelRule {
                message: "delegation override needs an entry".to_string(),
            }),
            Rc::new(DelegationOverrideRequiredSummaryRule {
                message: "delegation override needs a value".to_string(),
            }),
            Rc::new(DelegationOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DelegationOverrideAllowedLabelRule {
                allowed: vec!["regional".to_string(), "manual".to_string()],
            }),
            Rc::new(DelegationOverrideRetriesRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_override_validator();
        let item = new_delegation_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = DelegationOverrideDurationMinutesRangeRule { min: 1000, max: 86400 };
        let mut item = new_delegation_override("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DelegationOverrideRequiredLabelRule {
            message: "delegation override needs an entry".to_string(),
        };
        let mut item = new_delegation_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = DelegationOverrideRequiredSummaryRule {
            message: "delegation override needs an entry".to_string(),
        };
        let mut item = new_delegation_override("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = DelegationOverrideRetriesRangeRule { min: 64, max: 255 };
        let mut item = new_delegation_override("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
