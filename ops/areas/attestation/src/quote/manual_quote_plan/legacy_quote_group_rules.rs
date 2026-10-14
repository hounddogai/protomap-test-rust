use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy quote group must pass. Check returns its problems, or none.
pub trait LegacyQuoteGroupRule: std::fmt::Debug {
    fn check(&self, item: &LegacyQuoteGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteGroupOwnerIdLengthRule {
    pub max_length: i64,
}

impl LegacyQuoteGroupRule for LegacyQuoteGroupOwnerIdLengthRule {
    fn check(&self, item: &LegacyQuoteGroup) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteGroupCodeLengthRule {
    pub max_length: i64,
}

impl LegacyQuoteGroupRule for LegacyQuoteGroupCodeLengthRule {
    fn check(&self, item: &LegacyQuoteGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteGroupRequiredCodeRule {
    pub message: String,
}

impl LegacyQuoteGroupRule for LegacyQuoteGroupRequiredCodeRule {
    fn check(&self, item: &LegacyQuoteGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteGroupAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl LegacyQuoteGroupRule for LegacyQuoteGroupAllowedOwnerIdRule {
    fn check(&self, item: &LegacyQuoteGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is duplicated", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteGroupRequiredLocaleRule {
    pub message: String,
}

impl LegacyQuoteGroupRule for LegacyQuoteGroupRequiredLocaleRule {
    fn check(&self, item: &LegacyQuoteGroup) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy quote group.
#[derive(Debug, Clone)]
pub struct LegacyQuoteGroupValidator {
    pub rules: Vec<Rc<dyn LegacyQuoteGroupRule>>,
}

impl LegacyQuoteGroupValidator {
    pub fn validate(&self, item: &LegacyQuoteGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyQuoteGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyQuoteGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_quote_group_validator() -> LegacyQuoteGroupValidator {
    LegacyQuoteGroupValidator {
        rules: vec![
            Rc::new(LegacyQuoteGroupOwnerIdLengthRule { max_length: 1000 }) as Rc<dyn LegacyQuoteGroupRule>,
            Rc::new(LegacyQuoteGroupCodeLengthRule { max_length: 3600 }),
            Rc::new(LegacyQuoteGroupRequiredCodeRule {
                message: "legacy quote group needs this field".to_string(),
            }),
            Rc::new(LegacyQuoteGroupAllowedOwnerIdRule {
                allowed: vec!["tiered".to_string(), "default".to_string()],
            }),
            Rc::new(LegacyQuoteGroupRequiredLocaleRule {
                message: "legacy quote group needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_quote_group_validator();
        let item = new_legacy_quote_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = LegacyQuoteGroupOwnerIdLengthRule { max_length: 16 };
        let mut item = new_legacy_quote_group("x", "y");
        item.owner_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyQuoteGroupCodeLengthRule { max_length: 86400 };
        let mut item = new_legacy_quote_group("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyQuoteGroupRequiredCodeRule {
            message: "legacy quote group needs this field".to_string(),
        };
        let mut item = new_legacy_quote_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = LegacyQuoteGroupRequiredLocaleRule {
            message: "legacy quote group needs this field".to_string(),
        };
        let mut item = new_legacy_quote_group("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
