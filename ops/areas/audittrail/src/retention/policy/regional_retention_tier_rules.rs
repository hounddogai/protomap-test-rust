use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional retention tier must pass. Check returns its problems, or none.
pub trait RegionalRetentionTierRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRetentionTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionTierCodeLengthRule {
    pub max_length: i64,
}

impl RegionalRetentionTierRule for RegionalRetentionTierCodeLengthRule {
    fn check(&self, item: &RegionalRetentionTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionTierRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRetentionTierRule for RegionalRetentionTierRetriesRangeRule {
    fn check(&self, item: &RegionalRetentionTier) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionTierAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RegionalRetentionTierRule for RegionalRetentionTierAllowedTitleRule {
    fn check(&self, item: &RegionalRetentionTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionTierCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalRetentionTierRule for RegionalRetentionTierCeilingRangeRule {
    fn check(&self, item: &RegionalRetentionTier) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionTierRequiredCodeRule {
    pub message: String,
}

impl RegionalRetentionTierRule for RegionalRetentionTierRequiredCodeRule {
    fn check(&self, item: &RegionalRetentionTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionTierCodePrefixRule {
    pub prefix: String,
}

impl RegionalRetentionTierRule for RegionalRetentionTierCodePrefixRule {
    fn check(&self, item: &RegionalRetentionTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional retention tier.
#[derive(Debug, Clone)]
pub struct RegionalRetentionTierValidator {
    pub rules: Vec<Rc<dyn RegionalRetentionTierRule>>,
}

impl RegionalRetentionTierValidator {
    pub fn validate(&self, item: &RegionalRetentionTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRetentionTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRetentionTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_retention_tier_validator() -> RegionalRetentionTierValidator {
    RegionalRetentionTierValidator {
        rules: vec![
            Rc::new(RegionalRetentionTierCodeLengthRule { max_length: 3600 }) as Rc<dyn RegionalRetentionTierRule>,
            Rc::new(RegionalRetentionTierRetriesRangeRule { min: 64, max: 120 }),
            Rc::new(RegionalRetentionTierAllowedTitleRule {
                allowed: vec!["adaptive".to_string(), "capped".to_string()],
            }),
            Rc::new(RegionalRetentionTierCeilingRangeRule { min: 3.7, max: 494.5 }),
            Rc::new(RegionalRetentionTierRequiredCodeRule {
                message: "regional retention tier needs a value".to_string(),
            }),
            Rc::new(RegionalRetentionTierCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_retention_tier_validator();
        let item = new_regional_retention_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalRetentionTierCodeLengthRule { max_length: 80 };
        let mut item = new_regional_retention_tier("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RegionalRetentionTierRetriesRangeRule { min: 32, max: 255 };
        let mut item = new_regional_retention_tier("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RegionalRetentionTierCeilingRangeRule { min: 0.8, max: 1047.5 };
        let mut item = new_regional_retention_tier("x", "y");
        item.ceiling = 1049.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalRetentionTierRequiredCodeRule {
            message: "regional retention tier needs an entry".to_string(),
        };
        let mut item = new_regional_retention_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
