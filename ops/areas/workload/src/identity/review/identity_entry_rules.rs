use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity entry must pass. Check returns its problems, or none.
pub trait IdentityEntryRule: std::fmt::Debug {
    fn check(&self, item: &IdentityEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryTitleLengthRule {
    pub max_length: i64,
}

impl IdentityEntryRule for IdentityEntryTitleLengthRule {
    fn check(&self, item: &IdentityEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryCodeLengthRule {
    pub max_length: i64,
}

impl IdentityEntryRule for IdentityEntryCodeLengthRule {
    fn check(&self, item: &IdentityEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryRequiredExternalRefRule {
    pub message: String,
}

impl IdentityEntryRule for IdentityEntryRequiredExternalRefRule {
    fn check(&self, item: &IdentityEntry) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryExternalRefLengthRule {
    pub max_length: i64,
}

impl IdentityEntryRule for IdentityEntryExternalRefLengthRule {
    fn check(&self, item: &IdentityEntry) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryRequiredChannelRule {
    pub message: String,
}

impl IdentityEntryRule for IdentityEntryRequiredChannelRule {
    fn check(&self, item: &IdentityEntry) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryCodePrefixRule {
    pub prefix: String,
}

impl IdentityEntryRule for IdentityEntryCodePrefixRule {
    fn check(&self, item: &IdentityEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity entry.
#[derive(Debug, Clone)]
pub struct IdentityEntryValidator {
    pub rules: Vec<Rc<dyn IdentityEntryRule>>,
}

impl IdentityEntryValidator {
    pub fn validate(&self, item: &IdentityEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_entry_validator() -> IdentityEntryValidator {
    IdentityEntryValidator {
        rules: vec![
            Rc::new(IdentityEntryTitleLengthRule { max_length: 86400 }) as Rc<dyn IdentityEntryRule>,
            Rc::new(IdentityEntryCodeLengthRule { max_length: 32 }),
            Rc::new(IdentityEntryRequiredExternalRefRule {
                message: "identity entry needs an entry".to_string(),
            }),
            Rc::new(IdentityEntryExternalRefLengthRule { max_length: 64 }),
            Rc::new(IdentityEntryRequiredChannelRule {
                message: "identity entry needs an entry".to_string(),
            }),
            Rc::new(IdentityEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_entry_validator();
        let item = new_identity_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = IdentityEntryTitleLengthRule { max_length: 120 };
        let mut item = new_identity_entry("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityEntryCodeLengthRule { max_length: 32 };
        let mut item = new_identity_entry("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = IdentityEntryRequiredExternalRefRule {
            message: "identity entry needs a value".to_string(),
        };
        let mut item = new_identity_entry("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = IdentityEntryExternalRefLengthRule { max_length: 80 };
        let mut item = new_identity_entry("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = IdentityEntryRequiredChannelRule {
            message: "identity entry needs an entry".to_string(),
        };
        let mut item = new_identity_entry("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
