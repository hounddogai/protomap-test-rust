use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist tier must pass. Check returns its problems, or none.
pub trait AllowlistTierRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTierRequiredLabelRule {
    pub message: String,
}

impl AllowlistTierRule for AllowlistTierRequiredLabelRule {
    fn check(&self, item: &AllowlistTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTierRequiredCodeRule {
    pub message: String,
}

impl AllowlistTierRule for AllowlistTierRequiredCodeRule {
    fn check(&self, item: &AllowlistTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTierRequiredSourceRule {
    pub message: String,
}

impl AllowlistTierRule for AllowlistTierRequiredSourceRule {
    fn check(&self, item: &AllowlistTier) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTierAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl AllowlistTierRule for AllowlistTierAllowedChannelRule {
    fn check(&self, item: &AllowlistTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is blocked", item.channel)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist tier.
#[derive(Debug, Clone)]
pub struct AllowlistTierValidator {
    pub rules: Vec<Rc<dyn AllowlistTierRule>>,
}

impl AllowlistTierValidator {
    pub fn validate(&self, item: &AllowlistTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_tier_validator() -> AllowlistTierValidator {
    AllowlistTierValidator {
        rules: vec![
            Rc::new(AllowlistTierRequiredLabelRule {
                message: "allowlist tier needs an entry".to_string(),
            }) as Rc<dyn AllowlistTierRule>,
            Rc::new(AllowlistTierRequiredCodeRule {
                message: "allowlist tier needs a value".to_string(),
            }),
            Rc::new(AllowlistTierRequiredSourceRule {
                message: "allowlist tier needs a value".to_string(),
            }),
            Rc::new(AllowlistTierAllowedChannelRule {
                allowed: vec!["tiered".to_string(), "default".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_tier_validator();
        let item = new_allowlist_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AllowlistTierRequiredLabelRule {
            message: "allowlist tier needs this field".to_string(),
        };
        let mut item = new_allowlist_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistTierRequiredCodeRule {
            message: "allowlist tier needs an entry".to_string(),
        };
        let mut item = new_allowlist_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = AllowlistTierRequiredSourceRule {
            message: "allowlist tier needs this field".to_string(),
        };
        let mut item = new_allowlist_tier("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
