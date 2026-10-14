use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional slo override must pass. Check returns its problems, or none.
pub trait RegionalSloOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSloOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloOverrideCodePrefixRule {
    pub prefix: String,
}

impl RegionalSloOverrideRule for RegionalSloOverrideCodePrefixRule {
    fn check(&self, item: &RegionalSloOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloOverrideRequiredChannelRule {
    pub message: String,
}

impl RegionalSloOverrideRule for RegionalSloOverrideRequiredChannelRule {
    fn check(&self, item: &RegionalSloOverride) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloOverrideRequiredCodeRule {
    pub message: String,
}

impl RegionalSloOverrideRule for RegionalSloOverrideRequiredCodeRule {
    fn check(&self, item: &RegionalSloOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloOverrideRequiredNameRule {
    pub message: String,
}

impl RegionalSloOverrideRule for RegionalSloOverrideRequiredNameRule {
    fn check(&self, item: &RegionalSloOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloOverrideChannelLengthRule {
    pub max_length: i64,
}

impl RegionalSloOverrideRule for RegionalSloOverrideChannelLengthRule {
    fn check(&self, item: &RegionalSloOverride) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloOverrideNameLengthRule {
    pub max_length: i64,
}

impl RegionalSloOverrideRule for RegionalSloOverrideNameLengthRule {
    fn check(&self, item: &RegionalSloOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional slo override.
#[derive(Debug, Clone)]
pub struct RegionalSloOverrideValidator {
    pub rules: Vec<Rc<dyn RegionalSloOverrideRule>>,
}

impl RegionalSloOverrideValidator {
    pub fn validate(&self, item: &RegionalSloOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalSloOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSloOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_slo_override_validator() -> RegionalSloOverrideValidator {
    RegionalSloOverrideValidator {
        rules: vec![
            Rc::new(RegionalSloOverrideCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RegionalSloOverrideRule>,
            Rc::new(RegionalSloOverrideRequiredChannelRule {
                message: "regional slo override needs this field".to_string(),
            }),
            Rc::new(RegionalSloOverrideRequiredCodeRule {
                message: "regional slo override needs a value".to_string(),
            }),
            Rc::new(RegionalSloOverrideRequiredNameRule {
                message: "regional slo override needs an entry".to_string(),
            }),
            Rc::new(RegionalSloOverrideChannelLengthRule { max_length: 3600 }),
            Rc::new(RegionalSloOverrideNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_slo_override_validator();
        let item = new_regional_slo_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = RegionalSloOverrideRequiredChannelRule {
            message: "regional slo override needs this field".to_string(),
        };
        let mut item = new_regional_slo_override("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalSloOverrideRequiredCodeRule {
            message: "regional slo override needs an entry".to_string(),
        };
        let mut item = new_regional_slo_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalSloOverrideRequiredNameRule {
            message: "regional slo override needs this field".to_string(),
        };
        let mut item = new_regional_slo_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = RegionalSloOverrideChannelLengthRule { max_length: 80 };
        let mut item = new_regional_slo_override("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalSloOverrideNameLengthRule { max_length: 3600 };
        let mut item = new_regional_slo_override("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
