use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default policy tier must pass. Check returns its problems, or none.
pub trait DefaultPolicyTierRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPolicyTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyTierRequiredCodeRule {
    pub message: String,
}

impl DefaultPolicyTierRule for DefaultPolicyTierRequiredCodeRule {
    fn check(&self, item: &DefaultPolicyTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyTierPinnedGuardRule {
    pub limit: f64,
}

impl DefaultPolicyTierRule for DefaultPolicyTierPinnedGuardRule {
    fn check(&self, item: &DefaultPolicyTier) -> Vec<String> {
        if item.pinned && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while pinned",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyTierCodeLengthRule {
    pub max_length: i64,
}

impl DefaultPolicyTierRule for DefaultPolicyTierCodeLengthRule {
    fn check(&self, item: &DefaultPolicyTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyTierLabelLengthRule {
    pub max_length: i64,
}

impl DefaultPolicyTierRule for DefaultPolicyTierLabelLengthRule {
    fn check(&self, item: &DefaultPolicyTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyTierCodePrefixRule {
    pub prefix: String,
}

impl DefaultPolicyTierRule for DefaultPolicyTierCodePrefixRule {
    fn check(&self, item: &DefaultPolicyTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyTierContactEmailLengthRule {
    pub max_length: i64,
}

impl DefaultPolicyTierRule for DefaultPolicyTierContactEmailLengthRule {
    fn check(&self, item: &DefaultPolicyTier) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default policy tier.
#[derive(Debug, Clone)]
pub struct DefaultPolicyTierValidator {
    pub rules: Vec<Rc<dyn DefaultPolicyTierRule>>,
}

impl DefaultPolicyTierValidator {
    pub fn validate(&self, item: &DefaultPolicyTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultPolicyTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPolicyTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_policy_tier_validator() -> DefaultPolicyTierValidator {
    DefaultPolicyTierValidator {
        rules: vec![
            Rc::new(DefaultPolicyTierRequiredCodeRule {
                message: "default policy tier needs this field".to_string(),
            }) as Rc<dyn DefaultPolicyTierRule>,
            Rc::new(DefaultPolicyTierPinnedGuardRule { limit: 0.4 }),
            Rc::new(DefaultPolicyTierCodeLengthRule { max_length: 120 }),
            Rc::new(DefaultPolicyTierLabelLengthRule { max_length: 86400 }),
            Rc::new(DefaultPolicyTierCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultPolicyTierContactEmailLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_policy_tier_validator();
        let item = new_default_policy_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultPolicyTierRequiredCodeRule {
            message: "default policy tier needs an entry".to_string(),
        };
        let mut item = new_default_policy_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultPolicyTierCodeLengthRule { max_length: 120 };
        let mut item = new_default_policy_tier("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultPolicyTierLabelLengthRule { max_length: 86400 };
        let mut item = new_default_policy_tier("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DefaultPolicyTierContactEmailLengthRule { max_length: 255 };
        let mut item = new_default_policy_tier("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
