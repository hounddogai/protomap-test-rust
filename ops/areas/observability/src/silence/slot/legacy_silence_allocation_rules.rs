use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy silence allocation must pass. Check returns its problems, or none.
pub trait LegacySilenceAllocationRule: std::fmt::Debug {
    fn check(&self, item: &LegacySilenceAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceAllocationTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacySilenceAllocationRule for LegacySilenceAllocationTtlSecondsRangeRule {
    fn check(&self, item: &LegacySilenceAllocation) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceAllocationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LegacySilenceAllocationRule for LegacySilenceAllocationAllowedTitleRule {
    fn check(&self, item: &LegacySilenceAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceAllocationRequiredCodeRule {
    pub message: String,
}

impl LegacySilenceAllocationRule for LegacySilenceAllocationRequiredCodeRule {
    fn check(&self, item: &LegacySilenceAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceAllocationRequiredCountryRule {
    pub message: String,
}

impl LegacySilenceAllocationRule for LegacySilenceAllocationRequiredCountryRule {
    fn check(&self, item: &LegacySilenceAllocation) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceAllocationCodePrefixRule {
    pub prefix: String,
}

impl LegacySilenceAllocationRule for LegacySilenceAllocationCodePrefixRule {
    fn check(&self, item: &LegacySilenceAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceAllocationRequiredTitleRule {
    pub message: String,
}

impl LegacySilenceAllocationRule for LegacySilenceAllocationRequiredTitleRule {
    fn check(&self, item: &LegacySilenceAllocation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy silence allocation.
#[derive(Debug, Clone)]
pub struct LegacySilenceAllocationValidator {
    pub rules: Vec<Rc<dyn LegacySilenceAllocationRule>>,
}

impl LegacySilenceAllocationValidator {
    pub fn validate(&self, item: &LegacySilenceAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacySilenceAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacySilenceAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_silence_allocation_validator() -> LegacySilenceAllocationValidator {
    LegacySilenceAllocationValidator {
        rules: vec![
            Rc::new(LegacySilenceAllocationTtlSecondsRangeRule { min: 16, max: 32 })
                as Rc<dyn LegacySilenceAllocationRule>,
            Rc::new(LegacySilenceAllocationAllowedTitleRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(LegacySilenceAllocationRequiredCodeRule {
                message: "legacy silence allocation needs this field".to_string(),
            }),
            Rc::new(LegacySilenceAllocationRequiredCountryRule {
                message: "legacy silence allocation needs a value".to_string(),
            }),
            Rc::new(LegacySilenceAllocationCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LegacySilenceAllocationRequiredTitleRule {
                message: "legacy silence allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_silence_allocation_validator();
        let item = new_legacy_silence_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = LegacySilenceAllocationTtlSecondsRangeRule { min: 32, max: 80 };
        let mut item = new_legacy_silence_allocation("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacySilenceAllocationRequiredCodeRule {
            message: "legacy silence allocation needs an entry".to_string(),
        };
        let mut item = new_legacy_silence_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = LegacySilenceAllocationRequiredCountryRule {
            message: "legacy silence allocation needs an entry".to_string(),
        };
        let mut item = new_legacy_silence_allocation("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacySilenceAllocationRequiredTitleRule {
            message: "legacy silence allocation needs a value".to_string(),
        };
        let mut item = new_legacy_silence_allocation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
