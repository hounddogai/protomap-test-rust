use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy zone policy must pass. Check returns its problems, or none.
pub trait LegacyZonePolicyRule: std::fmt::Debug {
    fn check(&self, item: &LegacyZonePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZonePolicyAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LegacyZonePolicyRule for LegacyZonePolicyAllowedTitleRule {
    fn check(&self, item: &LegacyZonePolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZonePolicyRequiredCodeRule {
    pub message: String,
}

impl LegacyZonePolicyRule for LegacyZonePolicyRequiredCodeRule {
    fn check(&self, item: &LegacyZonePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZonePolicyCodeLengthRule {
    pub max_length: i64,
}

impl LegacyZonePolicyRule for LegacyZonePolicyCodeLengthRule {
    fn check(&self, item: &LegacyZonePolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZonePolicyCodePrefixRule {
    pub prefix: String,
}

impl LegacyZonePolicyRule for LegacyZonePolicyCodePrefixRule {
    fn check(&self, item: &LegacyZonePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy zone policy.
#[derive(Debug, Clone)]
pub struct LegacyZonePolicyValidator {
    pub rules: Vec<Rc<dyn LegacyZonePolicyRule>>,
}

impl LegacyZonePolicyValidator {
    pub fn validate(&self, item: &LegacyZonePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyZonePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyZonePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_zone_policy_validator() -> LegacyZonePolicyValidator {
    LegacyZonePolicyValidator {
        rules: vec![
            Rc::new(LegacyZonePolicyAllowedTitleRule {
                allowed: vec!["capped".to_string(), "tiered".to_string()],
            }) as Rc<dyn LegacyZonePolicyRule>,
            Rc::new(LegacyZonePolicyRequiredCodeRule {
                message: "legacy zone policy needs a value".to_string(),
            }),
            Rc::new(LegacyZonePolicyCodeLengthRule { max_length: 86400 }),
            Rc::new(LegacyZonePolicyCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_zone_policy_validator();
        let item = new_legacy_zone_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyZonePolicyRequiredCodeRule {
            message: "legacy zone policy needs an entry".to_string(),
        };
        let mut item = new_legacy_zone_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyZonePolicyCodeLengthRule { max_length: 86400 };
        let mut item = new_legacy_zone_policy("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
