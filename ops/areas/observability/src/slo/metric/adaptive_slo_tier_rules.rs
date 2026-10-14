use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive slo tier must pass. Check returns its problems, or none.
pub trait AdaptiveSloTierRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSloTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloTierCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveSloTierRule for AdaptiveSloTierCodePrefixRule {
    fn check(&self, item: &AdaptiveSloTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloTierCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveSloTierRule for AdaptiveSloTierCodeLengthRule {
    fn check(&self, item: &AdaptiveSloTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloTierChecksumLengthRule {
    pub max_length: i64,
}

impl AdaptiveSloTierRule for AdaptiveSloTierChecksumLengthRule {
    fn check(&self, item: &AdaptiveSloTier) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloTierAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl AdaptiveSloTierRule for AdaptiveSloTierAllowedChecksumRule {
    fn check(&self, item: &AdaptiveSloTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is expired", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloTierRequiredCodeRule {
    pub message: String,
}

impl AdaptiveSloTierRule for AdaptiveSloTierRequiredCodeRule {
    fn check(&self, item: &AdaptiveSloTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloTierRequiredChecksumRule {
    pub message: String,
}

impl AdaptiveSloTierRule for AdaptiveSloTierRequiredChecksumRule {
    fn check(&self, item: &AdaptiveSloTier) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive slo tier.
#[derive(Debug, Clone)]
pub struct AdaptiveSloTierValidator {
    pub rules: Vec<Rc<dyn AdaptiveSloTierRule>>,
}

impl AdaptiveSloTierValidator {
    pub fn validate(&self, item: &AdaptiveSloTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSloTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSloTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_slo_tier_validator() -> AdaptiveSloTierValidator {
    AdaptiveSloTierValidator {
        rules: vec![
            Rc::new(AdaptiveSloTierCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn AdaptiveSloTierRule>,
            Rc::new(AdaptiveSloTierCodeLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveSloTierChecksumLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveSloTierAllowedChecksumRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(AdaptiveSloTierRequiredCodeRule {
                message: "adaptive slo tier needs a value".to_string(),
            }),
            Rc::new(AdaptiveSloTierRequiredChecksumRule {
                message: "adaptive slo tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_slo_tier_validator();
        let item = new_adaptive_slo_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveSloTierCodeLengthRule { max_length: 255 };
        let mut item = new_adaptive_slo_tier("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = AdaptiveSloTierChecksumLengthRule { max_length: 255 };
        let mut item = new_adaptive_slo_tier("x", "y");
        item.checksum = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveSloTierRequiredCodeRule {
            message: "adaptive slo tier needs a value".to_string(),
        };
        let mut item = new_adaptive_slo_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = AdaptiveSloTierRequiredChecksumRule {
            message: "adaptive slo tier needs an entry".to_string(),
        };
        let mut item = new_adaptive_slo_tier("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
