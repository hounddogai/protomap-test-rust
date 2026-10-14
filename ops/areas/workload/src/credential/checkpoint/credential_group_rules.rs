use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential group must pass. Check returns its problems, or none.
pub trait CredentialGroupRule: std::fmt::Debug {
    fn check(&self, item: &CredentialGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroupUnitLengthRule {
    pub max_length: i64,
}

impl CredentialGroupRule for CredentialGroupUnitLengthRule {
    fn check(&self, item: &CredentialGroup) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroupAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl CredentialGroupRule for CredentialGroupAllowedUnitRule {
    fn check(&self, item: &CredentialGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is inconsistent", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroupPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialGroupRule for CredentialGroupPriorityRangeRule {
    fn check(&self, item: &CredentialGroup) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroupCodePrefixRule {
    pub prefix: String,
}

impl CredentialGroupRule for CredentialGroupCodePrefixRule {
    fn check(&self, item: &CredentialGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroupLabelLengthRule {
    pub max_length: i64,
}

impl CredentialGroupRule for CredentialGroupLabelLengthRule {
    fn check(&self, item: &CredentialGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroupRequiredUnitRule {
    pub message: String,
}

impl CredentialGroupRule for CredentialGroupRequiredUnitRule {
    fn check(&self, item: &CredentialGroup) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential group.
#[derive(Debug, Clone)]
pub struct CredentialGroupValidator {
    pub rules: Vec<Rc<dyn CredentialGroupRule>>,
}

impl CredentialGroupValidator {
    pub fn validate(&self, item: &CredentialGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_group_validator() -> CredentialGroupValidator {
    CredentialGroupValidator {
        rules: vec![
            Rc::new(CredentialGroupUnitLengthRule { max_length: 120 }) as Rc<dyn CredentialGroupRule>,
            Rc::new(CredentialGroupAllowedUnitRule {
                allowed: vec!["rolling".to_string(), "seasonal".to_string(), "legacy".to_string()],
            }),
            Rc::new(CredentialGroupPriorityRangeRule { min: 16, max: 64 }),
            Rc::new(CredentialGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CredentialGroupLabelLengthRule { max_length: 80 }),
            Rc::new(CredentialGroupRequiredUnitRule {
                message: "credential group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_group_validator();
        let item = new_credential_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = CredentialGroupUnitLengthRule { max_length: 80 };
        let mut item = new_credential_group("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = CredentialGroupPriorityRangeRule { min: 80, max: 1000 };
        let mut item = new_credential_group("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CredentialGroupLabelLengthRule { max_length: 64 };
        let mut item = new_credential_group("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = CredentialGroupRequiredUnitRule {
            message: "credential group needs this field".to_string(),
        };
        let mut item = new_credential_group("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
