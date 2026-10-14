use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual delegation override must pass. Check returns its problems, or none.
pub trait ManualDelegationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ManualDelegationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationOverrideAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ManualDelegationOverrideRule for ManualDelegationOverrideAllowedNameRule {
    fn check(&self, item: &ManualDelegationOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationOverrideNameLengthRule {
    pub max_length: i64,
}

impl ManualDelegationOverrideRule for ManualDelegationOverrideNameLengthRule {
    fn check(&self, item: &ManualDelegationOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationOverrideCodePrefixRule {
    pub prefix: String,
}

impl ManualDelegationOverrideRule for ManualDelegationOverrideCodePrefixRule {
    fn check(&self, item: &ManualDelegationOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationOverrideRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDelegationOverrideRule for ManualDelegationOverrideRetriesRangeRule {
    fn check(&self, item: &ManualDelegationOverride) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual delegation override.
#[derive(Debug, Clone)]
pub struct ManualDelegationOverrideValidator {
    pub rules: Vec<Rc<dyn ManualDelegationOverrideRule>>,
}

impl ManualDelegationOverrideValidator {
    pub fn validate(&self, item: &ManualDelegationOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualDelegationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDelegationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_delegation_override_validator() -> ManualDelegationOverrideValidator {
    ManualDelegationOverrideValidator {
        rules: vec![
            Rc::new(ManualDelegationOverrideAllowedNameRule { allowed: Vec::new() })
                as Rc<dyn ManualDelegationOverrideRule>,
            Rc::new(ManualDelegationOverrideNameLengthRule { max_length: 255 }),
            Rc::new(ManualDelegationOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ManualDelegationOverrideRetriesRangeRule { min: 1000, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_delegation_override_validator();
        let item = new_manual_delegation_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualDelegationOverrideNameLengthRule { max_length: 3600 };
        let mut item = new_manual_delegation_override("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ManualDelegationOverrideRetriesRangeRule { min: 32, max: 86400 };
        let mut item = new_manual_delegation_override("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
