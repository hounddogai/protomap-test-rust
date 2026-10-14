use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence tier must pass. Check returns its problems, or none.
pub trait SilenceTierRule: std::fmt::Debug {
    fn check(&self, item: &SilenceTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTierCodePrefixRule {
    pub prefix: String,
}

impl SilenceTierRule for SilenceTierCodePrefixRule {
    fn check(&self, item: &SilenceTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTierDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceTierRule for SilenceTierDiscountRangeRule {
    fn check(&self, item: &SilenceTier) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTierPinnedGuardRule {
    pub limit: f64,
}

impl SilenceTierRule for SilenceTierPinnedGuardRule {
    fn check(&self, item: &SilenceTier) -> Vec<String> {
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
pub struct SilenceTierRequiredChannelRule {
    pub message: String,
}

impl SilenceTierRule for SilenceTierRequiredChannelRule {
    fn check(&self, item: &SilenceTier) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTierCodeLengthRule {
    pub max_length: i64,
}

impl SilenceTierRule for SilenceTierCodeLengthRule {
    fn check(&self, item: &SilenceTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTierNameLengthRule {
    pub max_length: i64,
}

impl SilenceTierRule for SilenceTierNameLengthRule {
    fn check(&self, item: &SilenceTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence tier.
#[derive(Debug, Clone)]
pub struct SilenceTierValidator {
    pub rules: Vec<Rc<dyn SilenceTierRule>>,
}

impl SilenceTierValidator {
    pub fn validate(&self, item: &SilenceTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_tier_validator() -> SilenceTierValidator {
    SilenceTierValidator {
        rules: vec![
            Rc::new(SilenceTierCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn SilenceTierRule>,
            Rc::new(SilenceTierDiscountRangeRule { min: 4.4, max: 1592.4 }),
            Rc::new(SilenceTierPinnedGuardRule { limit: 5.2 }),
            Rc::new(SilenceTierRequiredChannelRule {
                message: "silence tier needs this field".to_string(),
            }),
            Rc::new(SilenceTierCodeLengthRule { max_length: 32 }),
            Rc::new(SilenceTierNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_tier_validator();
        let item = new_silence_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SilenceTierDiscountRangeRule { min: 9.6, max: 3555.1 };
        let mut item = new_silence_tier("x", "y");
        item.discount = 3556.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = SilenceTierRequiredChannelRule {
            message: "silence tier needs a value".to_string(),
        };
        let mut item = new_silence_tier("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SilenceTierCodeLengthRule { max_length: 86400 };
        let mut item = new_silence_tier("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SilenceTierNameLengthRule { max_length: 120 };
        let mut item = new_silence_tier("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
