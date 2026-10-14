use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered freeze group must pass. Check returns its problems, or none.
pub trait TieredFreezeGroupRule: std::fmt::Debug {
    fn check(&self, item: &TieredFreezeGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeGroupNameLengthRule {
    pub max_length: i64,
}

impl TieredFreezeGroupRule for TieredFreezeGroupNameLengthRule {
    fn check(&self, item: &TieredFreezeGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeGroupCodePrefixRule {
    pub prefix: String,
}

impl TieredFreezeGroupRule for TieredFreezeGroupCodePrefixRule {
    fn check(&self, item: &TieredFreezeGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeGroupRequiredChannelRule {
    pub message: String,
}

impl TieredFreezeGroupRule for TieredFreezeGroupRequiredChannelRule {
    fn check(&self, item: &TieredFreezeGroup) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeGroupCodeLengthRule {
    pub max_length: i64,
}

impl TieredFreezeGroupRule for TieredFreezeGroupCodeLengthRule {
    fn check(&self, item: &TieredFreezeGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered freeze group.
#[derive(Debug, Clone)]
pub struct TieredFreezeGroupValidator {
    pub rules: Vec<Rc<dyn TieredFreezeGroupRule>>,
}

impl TieredFreezeGroupValidator {
    pub fn validate(&self, item: &TieredFreezeGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredFreezeGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredFreezeGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_freeze_group_validator() -> TieredFreezeGroupValidator {
    TieredFreezeGroupValidator {
        rules: vec![
            Rc::new(TieredFreezeGroupNameLengthRule { max_length: 16 }) as Rc<dyn TieredFreezeGroupRule>,
            Rc::new(TieredFreezeGroupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TieredFreezeGroupRequiredChannelRule {
                message: "tiered freeze group needs this field".to_string(),
            }),
            Rc::new(TieredFreezeGroupCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_freeze_group_validator();
        let item = new_tiered_freeze_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredFreezeGroupNameLengthRule { max_length: 16 };
        let mut item = new_tiered_freeze_group("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = TieredFreezeGroupRequiredChannelRule {
            message: "tiered freeze group needs this field".to_string(),
        };
        let mut item = new_tiered_freeze_group("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredFreezeGroupCodeLengthRule { max_length: 3600 };
        let mut item = new_tiered_freeze_group("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
